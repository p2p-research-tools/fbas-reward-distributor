use fbas_analyzer::*;
use fbas_reward_distributor::*;

use structopt::StructOpt;
use strum::VariantNames;

use std::path::PathBuf;

/// Rank nodes of an FBAS and allocate rewards to them accordingly
#[derive(Debug, StructOpt)]
#[structopt(
    name = "fbas_reward_distributor",
    about = "Rank nodes of an FBAS and allocate rewards to them accordingly",
    author = "Charmaine Ndolo"
)]
struct Cli {
    #[structopt(subcommand)]
    subcommand: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    Rank(RankCmds),
    Distribute(DistCmds),
}

/// Rank only, do not compute a distribution
#[derive(Debug, StructOpt)]
#[structopt(author = "Charmaine Ndolo")]
struct RankCmds {
    /// Path to JSON file describing the FBAS in stellarbeat.org "nodes" format.
    /// Will use STDIN if omitted.
    nodes_path: Option<PathBuf>,

    /// Ranking algorithm to use.
    #[structopt(short = "a", long = "algorithm", possible_values = RankingAlgConfig::VARIANTS)]
    alg: RankingAlgConfig,

    /// Number of samples to use for approximation
    #[structopt(
        short = "s",
        long = "samples",
        required_if("alg", "RankingAlgConfig::ApproxPowerIndex")
    )]
    samples: Option<usize>,

    /// Prior to any analysis, filter out all nodes marked as `"active" == false` in the input
    /// nodes JSON (the one at `nodes_path`).
    #[structopt(short = "i", long = "ignore-inactive-nodes")]
    ignore_inactive_nodes: bool,

    /// Identify nodes by their public key.
    /// Default is to use node IDs corresponding to indices in the input file.
    #[structopt(short = "p", long = "pretty")]
    pks: bool,
}

/// Compute a distribution based on ranking according to selected algorithm
#[derive(Debug, StructOpt)]
#[structopt(author = "Charmaine Ndolo")]
struct DistCmds {
    /// Path to JSON file describing the FBAS in stellarbeat.org "nodes" format.
    /// Will use STDIN if omitted.
    nodes_path: Option<PathBuf>,

    /// Prior to any analysis, filter out all nodes marked as `"active" == false` in the input
    /// nodes JSON (the one at `nodes_path`).
    #[structopt(short = "i", long = "ignore-inactive-nodes")]
    ignore_inactive_nodes: bool,

    /// Amount to be shared among the nodes.
    #[structopt(short = "r", long = "reward", default_value = "1")]
    total_reward: f64,

    /// Ranking algorithm to use.
    #[structopt(short = "a", long = "algorithm", possible_values = RankingAlgConfig::VARIANTS)]
    alg: RankingAlgConfig,

    /// Number of samples to use for approximation
    #[structopt(short = "s", long = "samples", required_if("alg", "ApproxPowerIndex"))]
    samples: Option<usize>,

    /// Identify nodes by their public key.
    /// Default is to use node IDs corresponding to indices in the input file.
    #[structopt(short = "p", long = "pretty")]
    pks: bool,
}

#[derive(Debug, strum::EnumString, strum::EnumVariantNames)]
enum RankingAlgConfig {
    #[strum(serialize = "noderank")]
    NodeRank,
    #[strum(serialize = "exact-powerindex")]
    ExactPowerIndex,
    #[strum(serialize = "approx-powerindex")]
    ApproxPowerIndex,
}

fn extract_ranking_params(
    rank_cmd: RankCmds,
) -> (Option<PathBuf>, RankingAlgConfig, Option<usize>, bool, bool) {
    (
        rank_cmd.nodes_path,
        rank_cmd.alg,
        rank_cmd.samples,
        rank_cmd.ignore_inactive_nodes,
        rank_cmd.pks,
    )
}

fn extract_dist_params(
    dist_cmd: DistCmds,
) -> (
    Option<PathBuf>,
    RankingAlgConfig,
    Reward,
    Option<usize>,
    bool,
    bool,
) {
    (
        dist_cmd.nodes_path,
        dist_cmd.alg,
        dist_cmd.total_reward,
        dist_cmd.samples,
        dist_cmd.ignore_inactive_nodes,
        dist_cmd.pks,
    )
}

fn get_ranking_alg_from_params(cfg: RankingAlgConfig, samples: Option<usize>) -> RankingAlg {
    match cfg {
        RankingAlgConfig::NodeRank => RankingAlg::NodeRank,
        RankingAlgConfig::ExactPowerIndex => RankingAlg::ExactPowerIndex,
        RankingAlgConfig::ApproxPowerIndex => {
            if let Some(sample_size) = samples {
                RankingAlg::ApproxPowerIndex(sample_size)
            } else {
                panic!("-a approx-powerindex requires the number of samples.\n Add -s or --samples [num] to your command and try again.");
            }
        }
    }
}

fn main() {
    let cli = Cli::from_args();
    println!("cli args {:?}", cli.subcommand);
    let (rank, dist) = match cli.subcommand {
        Some(SubCommand::Rank(cmd)) => (Some(extract_ranking_params(cmd)), None),
        Some(SubCommand::Distribute(cmd)) => (None, Some(extract_dist_params(cmd))),
        None => {
            println!("Invalid command. Exiting..");
            return;
        }
    };
    if let Some(rank_cmd) = rank {
        let ignore_inactive_nodes = rank_cmd.3;
        let fbas = load_fbas(rank_cmd.0.as_ref(), ignore_inactive_nodes);
        let node_ids: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let alg_cfg = rank_cmd.1;
        let samples = rank_cmd.2;
        let alg = get_ranking_alg_from_params(alg_cfg, samples);
        let use_pks = rank_cmd.3;
        let rankings = compute_influence(&node_ids, &fbas, alg, use_pks);
        println!("List of Rankings as (NodeId, PK, Score):\n {:?}", rankings);
    } else if let Some(dist_cmd) = dist {
        let ignore_inactive_nodes = dist_cmd.4;
        let fbas = load_fbas(dist_cmd.0.as_ref(), ignore_inactive_nodes);
        let node_ids: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let alg_cfg = dist_cmd.1;
        let samples = dist_cmd.3;
        let alg = get_ranking_alg_from_params(alg_cfg, samples);
        let total_reward = dist_cmd.2;
        let use_pks = dist_cmd.4;
        let allocation = distribute_rewards(alg, &node_ids, &fbas, total_reward, use_pks);
        println!(
            "List of Distributions as (NodeId, PK, Score, Reward):\n {:?}",
            allocation
        );
    }
}

fn load_fbas(o_nodes_path: Option<&PathBuf>, ignore_inactive_nodes: bool) -> Fbas {
    let fbas = if let Some(nodes_path) = o_nodes_path {
        eprintln!("Reading FBAS JSON from file...");
        let mut fbas = Fbas::from_json_file(nodes_path);
        if ignore_inactive_nodes {
            let inactive_nodes =
                FilteredNodes::from_json_file(nodes_path, |v| v["active"] == false);
            fbas = fbas.without_nodes_pretty(&inactive_nodes.into_pretty_vec());
        }
        fbas
    } else {
        eprintln!("Reading FBAS JSON from STDIN...");
        if ignore_inactive_nodes {
            panic!(
                "Ignoring nodes is currently not supported when reading an FBAS from STDIN;
                perhaps filter the input yourself? (e.g., with `jq`)"
            );
        }
        Fbas::from_json_stdin()
    };
    eprintln!("Loaded FBAS with {} nodes.", fbas.number_of_nodes());
    fbas
}

/// Rank nodes using either S-S Power Index or NodeRank and return a sorted list of nodes
fn compute_influence(
    node_ids: &[NodeId],
    fbas: &Fbas,
    alg: RankingAlg,
    use_pks: bool,
) -> Vec<NodeRanking> {
    let rankings = rank_nodes(fbas, alg);
    create_node_ranking_report(node_ids, rankings, fbas, use_pks)
}

/// Distribute the reward between nodes based on their contribution as calculated by a ranking
/// algorithm and return a sorted list
fn distribute_rewards(
    algo: RankingAlg,
    nodes: &[NodeId],
    fbas: &Fbas,
    reward_value: f64,
    use_pks: bool,
) -> Vec<(NodeId, PublicKey, Score, Reward)> {
    let allocation = match algo {
        RankingAlg::NodeRank => graph_theory_distribution(nodes, fbas, reward_value),
        RankingAlg::ExactPowerIndex => exact_game_theory_distribution(fbas, reward_value),
        RankingAlg::ApproxPowerIndex(samples) => {
            println!("samples {}", samples);
            approx_game_theory_distribution(samples, fbas, reward_value)
        }
    };
    create_reward_report(allocation, fbas, use_pks)
}
