use fbas_reward_distributor::*;

use env_logger::Env;
use fbas_analyzer::Fbas;
use lazy_static::lazy_static;
use log::info;
use par_map::ParMap;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    io,
    path::PathBuf,
    sync::Mutex,
};
use structopt::StructOpt;

lazy_static! {
    static ref TRUTH_VALUES: Mutex<HashMap<usize, Vec<Score>>> = {
        let truth = HashMap::default();
        Mutex::new(truth)
    };
}

/// Run performance measurements on different sized FBASs based on the input parameters.
#[derive(Debug, StructOpt)]
#[structopt(
    name = "approximation_tests",
    about = "Run accuracy measurements on different sized FBASs based on the input parameters
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
    println!(
        "Starting measurements for {:?} like FBAS with upto {} nodes.\n
             Performing {} iterations per FBAS.",
        fbas_type, args.max_top_tier_size, args.runs
    );

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
) -> Result<BTreeMap<InputDataPoint, ErrorDataPoint>, Box<dyn Error>> {
    if let Some(path) = path {
        let data_points = read_error_data_csv_from_file(path)?;
        let data_points_map = data_points
            .into_iter()
            .map(|d| (InputDataPoint::from_error_data_point(&d), d))
            .collect();
        Ok(data_points_map)
    } else {
        Ok(BTreeMap::new())
    }
}

fn make_sorted_tasklist(
    inputs: Vec<InputDataPoint>,
    existing_outputs: BTreeMap<InputDataPoint, ErrorDataPoint>,
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
        .chain(existing_outputs.values().cloned().map(Task::ReuseErrorData))
        .collect();
    tasks.sort_by_cached_key(|t| t.label());
    tasks
}

fn bulk_do(
    tasks: Vec<Task>,
    jobs: usize,
    fbas_type: FbasType,
    qi_check: bool,
) -> impl Iterator<Item = ErrorDataPoint> {
    tasks
        .into_iter()
        .with_nb_threads(jobs)
        .par_map(move |task| analyze_or_reuse(task, fbas_type.clone(), qi_check))
}

fn analyze_or_reuse(task: Task, fbas_type: FbasType, qi_check: bool) -> ErrorDataPoint {
    match task {
        Task::ReuseErrorData(output) => {
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

fn get_or_compute_truth_value(fbas_size: usize, fbas: &Fbas, qi_check: bool) -> Vec<Score> {
    let cache_scores = get_scores_from_cache(fbas_size);

    let exact_scores = if let Some(scores) = cache_scores {
        info!("Found power index scores for {} nodes in cache.", fbas_size);
        scores
    } else {
        info!("Computing PowerIndexEnum for FBAS with {} nodes", fbas_size);
        let exact_power_index = rank_nodes(fbas, RankingAlg::PowerIndexEnum(None), qi_check);
        info!("Completed power index for FBAS of size {}.", fbas_size);
        add_to_cache(fbas_size, exact_power_index.clone());
        exact_power_index
    };
    exact_scores
}

fn rank(input: InputDataPoint, fbas_type: FbasType, qi_check: bool) -> ErrorDataPoint {
    let fbas = fbas_type.make_one(input.top_tier_size);
    assert!(fbas.number_of_nodes() == input.top_tier_size);
    let size = fbas.number_of_nodes();
    let exact_power_index = get_or_compute_truth_value(size, &fbas, qi_check);
    info!("Starting run {} for FBAS with {} nodes", input.run, size);
    info!(
        "Starting 10^1 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let approx_power_indices_10_pow_1 = rank_nodes(
        &fbas,
        RankingAlg::PowerIndexApprox(10usize.pow(1)),
        qi_check,
    );
    let (mean_abs_error_10_pow_1, median_abs_error_10_pow_1, mean_abs_percentage_error_10_pow_1) =
        mean_med_pctg_errors(&approx_power_indices_10_pow_1, &exact_power_index);
    info!("Completed 10^1 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^2 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let approx_power_indices_10_pow_2 = rank_nodes(
        &fbas,
        RankingAlg::PowerIndexApprox(10usize.pow(2)),
        qi_check,
    );
    let (mean_abs_error_10_pow_2, median_abs_error_10_pow_2, mean_abs_percentage_error_10_pow_2) =
        mean_med_pctg_errors(&approx_power_indices_10_pow_2, &exact_power_index);
    info!("Completed 10^2 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^3 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let approx_power_indices_10_pow_3 = rank_nodes(
        &fbas,
        RankingAlg::PowerIndexApprox(10usize.pow(3)),
        qi_check,
    );
    let (mean_abs_error_10_pow_3, median_abs_error_10_pow_3, mean_abs_percentage_error_10_pow_3) =
        mean_med_pctg_errors(&approx_power_indices_10_pow_3, &exact_power_index);
    info!("Completed 10^3 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^4 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let approx_power_indices_10_pow_4 = rank_nodes(
        &fbas,
        RankingAlg::PowerIndexApprox(10usize.pow(4)),
        qi_check,
    );
    let (mean_abs_error_10_pow_4, median_abs_error_10_pow_4, mean_abs_percentage_error_10_pow_4) =
        mean_med_pctg_errors(&approx_power_indices_10_pow_4, &exact_power_index);
    info!("Completed 10^4 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^5 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let approx_power_indices_10_pow_5 = rank_nodes(
        &fbas,
        RankingAlg::PowerIndexApprox(10usize.pow(5)),
        qi_check,
    );
    let (mean_abs_error_10_pow_5, median_abs_error_10_pow_5, mean_abs_percentage_error_10_pow_5) =
        mean_med_pctg_errors(&approx_power_indices_10_pow_5, &exact_power_index);
    info!("Completed 10^5 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^6 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let approx_power_indices_10_pow_6 = rank_nodes(
        &fbas,
        RankingAlg::PowerIndexApprox(10usize.pow(6)),
        qi_check,
    );
    let (mean_abs_error_10_pow_6, median_abs_error_10_pow_6, mean_abs_percentage_error_10_pow_6) =
        mean_med_pctg_errors(&approx_power_indices_10_pow_6, &exact_power_index);
    info!("Completed 10^6 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^7 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let approx_power_indices_10_pow_7 = rank_nodes(
        &fbas,
        RankingAlg::PowerIndexApprox(10usize.pow(7)),
        qi_check,
    );
    let (mean_abs_error_10_pow_7, median_abs_error_10_pow_7, mean_abs_percentage_error_10_pow_7) =
        mean_med_pctg_errors(&approx_power_indices_10_pow_7, &exact_power_index);
    info!("Completed 10^7 approximation for FBAS of size {}.", size);
    info!(
        "Starting 10^8 approximation run {} for FBAS of size {}.",
        input.run, size
    );
    let approx_power_indices_10_pow_8 = rank_nodes(
        &fbas,
        RankingAlg::PowerIndexApprox(10usize.pow(8)),
        qi_check,
    );
    let (mean_abs_error_10_pow_8, median_abs_error_10_pow_8, mean_abs_percentage_error_10_pow_8) =
        mean_med_pctg_errors(&approx_power_indices_10_pow_8, &exact_power_index);
    info!(
        "Completed 10^8 Approximation run {} for FBAS of size {}.",
        input.run, size
    );

    ErrorDataPoint {
        top_tier_size: input.top_tier_size,
        run: input.run,
        mean_abs_error_10_pow_1,
        median_abs_error_10_pow_1,
        mean_abs_percentage_error_10_pow_1,
        mean_abs_error_10_pow_2,
        median_abs_error_10_pow_2,
        mean_abs_percentage_error_10_pow_2,
        mean_abs_error_10_pow_3,
        median_abs_error_10_pow_3,
        mean_abs_percentage_error_10_pow_3,
        mean_abs_error_10_pow_4,
        median_abs_error_10_pow_4,
        mean_abs_percentage_error_10_pow_4,
        mean_abs_error_10_pow_5,
        median_abs_error_10_pow_5,
        mean_abs_percentage_error_10_pow_5,
        mean_abs_error_10_pow_6,
        median_abs_error_10_pow_6,
        mean_abs_percentage_error_10_pow_6,
        mean_abs_error_10_pow_7,
        median_abs_error_10_pow_7,
        mean_abs_percentage_error_10_pow_7,
        mean_abs_error_10_pow_8,
        median_abs_error_10_pow_8,
        mean_abs_percentage_error_10_pow_8,
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

fn get_scores_from_cache(fbas_size: usize) -> Option<Vec<Score>> {
    TRUTH_VALUES.lock().unwrap().get(&fbas_size).cloned()
}

fn add_to_cache(fbas_size: usize, scores: Vec<Score>) {
    TRUTH_VALUES.lock().unwrap().insert(fbas_size, scores);
}
