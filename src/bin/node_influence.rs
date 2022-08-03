use fbas_analyzer::{Fbas, NodeId};
use fbas_reward_distributor::*;

use structopt::StructOpt;

use std::path::PathBuf;

/// Rank nodes of an FBAS and allocate rewards to them accordingly
#[derive(Debug, StructOpt)]
#[structopt(
    name = "reward_distributor",
    about = "Rank nodes of an FBAS and allocate rewards to them accordingly",
    author = "Charmaine Ndolo"
)]
struct Cli {
    #[structopt(subcommand)]
    subcommand: SubCommand,
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
    /// Ranking algorithm to use.
    #[structopt(subcommand)]
    alg: RankingAlgConfig,

    /// Path to JSON file describing the FBAS in stellarbeat.org "nodes" format.
    /// Will use STDIN if omitted.
    nodes_path: Option<PathBuf>,

    /// Prior to any analysis, filter out all nodes marked as `"active" == false` in the input
    /// nodes JSON (the one at `nodes_path`).
    #[structopt(short = "i", long = "ignore-inactive-nodes")]
    ignore_inactive_nodes: bool,

    /// Identify nodes by their public key.
    /// Default is to use node IDs corresponding to indices in the input file.
    #[structopt(short = "p", long = "pretty")]
    pks: bool,

    /// Do not assert that the FBAS has quorum intersection before proceeding with further computations.
    /// Default behaviour is to always check for QI.
    #[structopt(short = "nq", long = "no-quorum-intersection")]
    dont_check_for_qi: bool,
}

/// Compute a distribution based on ranking according to selected algorithm
#[derive(Debug, StructOpt)]
#[structopt(author = "Charmaine Ndolo")]
struct DistCmds {
    /// Ranking algorithm to use.
    #[structopt(subcommand)]
    alg: RankingAlgConfig,

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

    /// Identify nodes by their public key.
    /// Default is to use node IDs corresponding to indices in the input file.
    #[structopt(short = "p", long = "pretty")]
    pks: bool,

    /// Do not assert that the FBAS has quorum intersection before proceeding with further computations.
    /// Default behaviour is to always check for QI.
    #[structopt(long = "no-quorum-intersection")]
    dont_check_for_qi: bool,
}

#[derive(Debug, StructOpt)]
enum RankingAlgConfig {
    /// Use NodeRank, an extension of PageRank, to measure nodes' weight in the FBAS
    NodeRank,
    /// Use Shapley-Shubik power indices to calculate nodes' importance in the FBAS. Not
    /// recommended for FBAS with many players because of time complexity
    /// The computation of minimal quorums can optionally be done before we start approximation.
    /// Useful, e.g. for timing measurements.
    PowerIndexEnum { exclude_tt_comp: Option<bool> },
    /// Approximate Shapley values as a measure of nodes' importance in the FBAS. The number of
    /// samples to use must be passed if selected.
    /// The computation of minimal quorums can optionally be done before we start approximation.
    /// Useful, e.g. for timing measurements.
    PowerIndexApprox { s: usize },
}

fn get_ranking_alg_from_params(cfg: RankingAlgConfig) -> RankingAlg {
    match cfg {
        RankingAlgConfig::NodeRank => RankingAlg::NodeRank,
        RankingAlgConfig::PowerIndexEnum { exclude_tt_comp } => {
            if let Some(true) = exclude_tt_comp {
                RankingAlg::PowerIndexEnum(Some(Vec::default()))
            } else {
                RankingAlg::PowerIndexEnum(None)
            }
        }
        RankingAlgConfig::PowerIndexApprox { s } => RankingAlg::PowerIndexApprox(s),
    }
}

#[allow(dead_code)]
fn get_top_tier_nodes(fbas: &Fbas, qi_check: bool) -> Vec<NodeId> {
    let min_qs = fbas_analyzer::find_minimal_quorums(fbas);
    if qi_check {
        println!("Ensuring the FBAS has quorum intersection.");
        assert!(
            fbas_analyzer::all_intersect(&min_qs),
            "FBAS lacks quorum intersection!"
        );
        println!("FBAS enjoys quorum intersection!");
    }
    let involved_nodes: Vec<NodeId> = fbas_analyzer::involved_nodes(&min_qs).into_iter().collect();
    println!("Computed top tier of size {}.", involved_nodes.len());
    involved_nodes
}

fn main() {
    let cli = Cli::from_args();
    match cli.subcommand {
        SubCommand::Rank(cmd) => {
            let ignore_inactive_nodes = cmd.ignore_inactive_nodes;
            let alg_cfg = cmd.alg;
            let use_pks = cmd.pks;
            let fbas = load_fbas(cmd.nodes_path.as_ref(), ignore_inactive_nodes);
            let node_ids: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
            let qi_check = !cmd.dont_check_for_qi;
            let alg = get_ranking_alg_from_params(alg_cfg);
            let rankings = compute_influence(&node_ids, &fbas, alg, use_pks, qi_check);
            println!("List of Rankings as (NodeId, PK, Score):\n {:?}", rankings);
        }
        SubCommand::Distribute(cmd) => {
            let ignore_inactive_nodes = cmd.ignore_inactive_nodes;
            let alg_cfg = cmd.alg;
            let total_reward = cmd.total_reward;
            let use_pks = cmd.pks;
            let fbas = load_fbas(cmd.nodes_path.as_ref(), ignore_inactive_nodes);
            let node_ids: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
            let qi_check = !cmd.dont_check_for_qi;
            let alg = get_ranking_alg_from_params(alg_cfg);
            let allocation =
                distribute_rewards(alg, &node_ids, &fbas, total_reward, use_pks, qi_check);
            println!(
                "List of Distributions as (NodeId, PK, Score, Reward):\n {:?}",
                allocation
            );
        }
    };
}

fn load_fbas(o_nodes_path: Option<&PathBuf>, ignore_inactive_nodes: bool) -> Fbas {
    let fbas = if let Some(nodes_path) = o_nodes_path {
        eprintln!("Reading FBAS JSON from file...");
        let mut fbas = Fbas::from_json_file(nodes_path);
        if ignore_inactive_nodes {
            let inactive_nodes =
                fbas_analyzer::FilteredNodes::from_json_file(nodes_path, |v| v["active"] == false);
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
    qi_check: bool,
) -> Vec<NodeRanking> {
    let rankings = rank_nodes(fbas, alg, qi_check);
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
    qi_check: bool,
) -> Vec<(NodeId, PublicKey, Score, Reward)> {
    let allocation = match algo {
        RankingAlg::NodeRank => graph_theory_distribution(nodes, fbas, reward_value, qi_check),
        RankingAlg::PowerIndexEnum(tt) => {
            exact_game_theory_distribution(fbas, reward_value, tt, qi_check)
        }
        RankingAlg::PowerIndexApprox(samples) => {
            approx_game_theory_distribution(samples, fbas, reward_value, qi_check)
        }
    };
    create_reward_report(allocation, fbas, use_pks)
}
