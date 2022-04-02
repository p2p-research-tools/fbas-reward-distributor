use fbas_analyzer::*;
use fbas_reward_distributor::*;

use env_logger::Env;
use log::{debug, info};
use par_map::ParMap;
use std::{collections::BTreeMap, error::Error, io, path::PathBuf};
use structopt::StructOpt;

/// Run performance measurements on different sized FBASs based on the input parameters.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "performance_tests",
    about = "Run performance measurements on different sized FBASs based on the input parameters
",
    author = "Charmaine Ndolo"
)]
struct Cli {
    /// Output CSV file (will output to STDOUT if omitted).
    #[structopt(short = "o", long = "out")]
    output_path: Option<PathBuf>,

    /// Largest FBAS to analyze, measured in number of top-tier nodes.
    #[structopt(short = "m", long = "max-top-tier-size")]
    max_top_tier_size: usize,

    #[structopt(subcommand)]
    fbas_type: FbasType,

    /// Update output file with missing results (doesn't repeat analyses for existing lines).
    #[structopt(short = "u", long = "update")]
    update: bool,

    /// Number of analysis runs per FBAS size.
    #[structopt(short = "r", long = "runs", default_value = "10")]
    runs: usize,

    /// Number of threads to use. Defaults to 1.
    #[structopt(short = "j", long = "jobs", default_value = "1")]
    jobs: usize,

    /// Do not assert that the FBAS has quorum intersection before proceeding with further computations.
    /// Default behaviour is to always check for QI.
    #[structopt(long = "no-quorum-intersection")]
    dont_check_for_qi: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env);
    let fbas_type = args.fbas_type;
    let inputs: Vec<InputDataPoint> =
        generate_inputs(args.max_top_tier_size, args.runs, fbas_type.clone());
    let existing_outputs = if args.update {
        load_existing_outputs(&args.output_path)?
    } else {
        BTreeMap::new()
    };
    let tasks = make_sorted_tasklist(inputs, existing_outputs);

    let qi_check = !args.dont_check_for_qi;
    let output_iterator = bulk_do(tasks, args.jobs, fbas_type.clone(), qi_check);
    println!("Starting performance measurements for {:?} like FBAS with upto {} nodes.\n Performing {} iterations per FBAS.",fbas_type, args.max_top_tier_size, args.runs);

    write_csv(output_iterator, &args.output_path, args.update)?;
    Ok(())
}

fn generate_inputs(
    max_top_tier_size: usize,
    runs: usize,
    fbas_type: FbasType,
) -> Vec<InputDataPoint> {
    let mut inputs = vec![];
    for top_tier_size in (1..max_top_tier_size + 1).filter(|m| m % fbas_type.node_increments() == 0)
    {
        for run in 0..runs {
            inputs.push(InputDataPoint { top_tier_size, run });
        }
    }
    inputs
}

fn load_existing_outputs(
    path: &Option<PathBuf>,
) -> Result<BTreeMap<InputDataPoint, PerfDataPoint>, Box<dyn Error>> {
    if let Some(path) = path {
        let data_points = read_csv_from_file(path)?;
        let data_points_map = data_points
            .into_iter()
            .map(|d| (InputDataPoint::from_perf_data_point(&d), d))
            .collect();
        Ok(data_points_map)
    } else {
        Ok(BTreeMap::new())
    }
}

fn make_sorted_tasklist(
    inputs: Vec<InputDataPoint>,
    existing_outputs: BTreeMap<InputDataPoint, PerfDataPoint>,
) -> Vec<Task> {
    let mut tasks: Vec<Task> = inputs
        .into_iter()
        .filter_map(|input| {
            if !existing_outputs.contains_key(&input) {
                Some(Task::Analyze(input))
            } else {
                None
            }
        })
        .chain(existing_outputs.values().cloned().map(Task::ReusePerfData))
        .collect();
    tasks.sort_by_cached_key(|t| t.label());
    tasks
}

fn bulk_do(
    tasks: Vec<Task>,
    jobs: usize,
    fbas_type: FbasType,
    qi_check: bool,
) -> impl Iterator<Item = PerfDataPoint> {
    tasks
        .into_iter()
        .with_nb_threads(jobs)
        .par_map(move |task| analyze_or_reuse(task, fbas_type.clone(), qi_check))
}

fn analyze_or_reuse(task: Task, fbas_type: FbasType, qi_check: bool) -> PerfDataPoint {
    match task {
        Task::ReusePerfData(output) => {
            eprintln!(
                "Reusing existing analysis results for m={}, run={}.",
                output.top_tier_size, output.run
            );
            output
        }
        Task::Analyze(input) => rank(input, fbas_type, qi_check),
        _ => panic!("Unexpected data point"),
    }
}

fn rank(input: InputDataPoint, fbas_type: FbasType, qi_check: bool) -> PerfDataPoint {
    let fbas = fbas_type.make_one(input.top_tier_size);
    assert!(fbas.number_of_nodes() == input.top_tier_size);
    let size = fbas.number_of_nodes();
    info!("Starting run {} for FBAS with {} nodes", input.run, size);
    info!(
        "Starting NodeRank run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_noderank) = timed_secs!(rank_nodes(&fbas, RankingAlg::NodeRank, qi_check));
    debug!(
        "Completed NodeRank run {} for FBAS of size {}.",
        input.run, size
    );
    info!(
        "Starting power index run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_exact_power_index) =
        timed_secs!(rank_nodes(&fbas, RankingAlg::ExactPowerIndex, qi_check));
    debug!(
        "Completed power index run {} for FBAS of size {}.",
        input.run, size
    );
    info!(
        "Starting 10^1 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_approx_power_indices_10_pow_1) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(1), None),
        qi_check
    ));
    info!("Completed 10^1 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^2 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_approx_power_indices_10_pow_2) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(2), None),
        qi_check
    ));
    info!("Completed 10^2 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^3 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_approx_power_indices_10_pow_3) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(3), None),
        qi_check
    ));
    info!("Completed 10^3 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^4 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_approx_power_indices_10_pow_4) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(4), None),
        qi_check
    ));
    info!("Completed 10^4 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^5 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_approx_power_indices_10_pow_5) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(5), None),
        qi_check
    ));
    info!("Completed 10^5 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^6 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_approx_power_indices_10_pow_6) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(6), None),
        qi_check
    ));
    info!("Completed 10^6 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^7 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_approx_power_indices_10_pow_7) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(7), None),
        qi_check
    ));
    info!("Completed 10^7 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^8 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_approx_power_indices_10_pow_8) = if input.top_tier_size <= 27 {
        timed_secs!(rank_nodes(
            &fbas,
            RankingAlg::ApproxPowerIndex(10usize.pow(8), None),
            qi_check
        ))
    } else {
        (Vec::default(), 0.0)
    };
    debug!("Completed 10^8 approximation for FBAS of size {}.", size);
    debug!(
        "Completed Approximation with TT computation run {} for FBAS of size {}.",
        input.run, size
    );
    let top_tier_nodes: Vec<NodeId> =
        fbas_analyzer::involved_nodes(&fbas_analyzer::find_minimal_quorums(&fbas))
            .iter()
            .collect();
    debug!(
        "Got top tier for current FBAS of {} nodes",
        fbas.number_of_nodes()
    );
    let (_, duration_after_mq_approx_power_indices_10_pow_1) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(1), Some(top_tier_nodes.clone())),
        qi_check
    ));
    info!(
        "Completed 10^1 approximation w/o TT for FBAS of size {}.",
        size
    );
    info!(
        "Starting 10^2 approximation w/o TT run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_after_mq_approx_power_indices_10_pow_2) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(2), Some(top_tier_nodes.clone())),
        qi_check
    ));
    info!(
        "Completed 10^2 approximation w/o TT for FBAS of size {}.",
        size
    );
    info!(
        "Starting 10^3 approximation w/o TT run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_after_mq_approx_power_indices_10_pow_3) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(3), Some(top_tier_nodes.clone())),
        qi_check
    ));
    info!(
        "Completed 10^3 approximation w/o TT for FBAS of size {}.",
        size
    );
    info!(
        "Starting 10^4 approximation w/o TT run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_after_mq_approx_power_indices_10_pow_4) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(4), Some(top_tier_nodes.clone())),
        qi_check
    ));
    info!(
        "Completed 10^4 approximation w/o TT for FBAS of size {}.",
        size
    );
    info!(
        "Starting 10^5 approximation w/o TT run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_after_mq_approx_power_indices_10_pow_5) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(5), Some(top_tier_nodes.clone())),
        qi_check
    ));
    info!(
        "Completed 10^5 approximation w/o TT for FBAS of size {}.",
        size
    );
    info!(
        "Starting 10^6 approximation w/o TT run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_after_mq_approx_power_indices_10_pow_6) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(6), Some(top_tier_nodes.clone())),
        qi_check
    ));
    info!(
        "Completed 10^6 approximation w/o TT for FBAS of size {}.",
        size
    );
    info!(
        "Starting 10^7 approximation w/o TT run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_after_mq_approx_power_indices_10_pow_7) = timed_secs!(rank_nodes(
        &fbas,
        RankingAlg::ApproxPowerIndex(10usize.pow(7), Some(top_tier_nodes.clone())),
        qi_check
    ));
    info!(
        "Completed 10^7 approximation w/o TT for FBAS of size {}.",
        size
    );
    info!(
        "Starting 10^8 approximation w/o TT run {} for FBAS of size {}.",
        input.run, size
    );
    let (_, duration_after_mq_approx_power_indices_10_pow_8) = if input.top_tier_size <= 27 {
        timed_secs!(rank_nodes(
            &fbas,
            RankingAlg::ApproxPowerIndex(10usize.pow(8), Some(top_tier_nodes)),
            qi_check
        ))
    } else {
        (Vec::default(), 0.0)
    };
    info!(
        "Completed 10^8 approximation with precomputed top tier for FBAS of size {}.",
        size
    );
    info!("Completed run {} for FBAS with {} nodes.", input.run, size);
    PerfDataPoint {
        top_tier_size: input.top_tier_size,
        run: input.run,
        duration_noderank,
        duration_exact_power_index,
        duration_approx_power_indices_10_pow_1,
        duration_approx_power_indices_10_pow_2,
        duration_approx_power_indices_10_pow_3,
        duration_approx_power_indices_10_pow_4,
        duration_approx_power_indices_10_pow_5,
        duration_approx_power_indices_10_pow_6,
        duration_approx_power_indices_10_pow_7,
        duration_approx_power_indices_10_pow_8,
        duration_after_mq_approx_power_indices_10_pow_1,
        duration_after_mq_approx_power_indices_10_pow_2,
        duration_after_mq_approx_power_indices_10_pow_3,
        duration_after_mq_approx_power_indices_10_pow_4,
        duration_after_mq_approx_power_indices_10_pow_5,
        duration_after_mq_approx_power_indices_10_pow_6,
        duration_after_mq_approx_power_indices_10_pow_7,
        duration_after_mq_approx_power_indices_10_pow_8,
    }
}

fn write_csv(
    data_points: impl IntoIterator<Item = impl serde::Serialize>,
    output_path: &Option<PathBuf>,
    overwrite_allowed: bool,
) -> Result<(), Box<dyn Error>> {
    if let Some(path) = output_path {
        if !overwrite_allowed && path.exists() {
            Err(Box::new(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Output file exists, refusing to overwrite.",
            )))
        } else {
            write_csv_to_file(data_points, path)
        }
    } else {
        write_csv_to_stdout(data_points)
    }
}
