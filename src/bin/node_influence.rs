use fbas_analyzer::*;
use fbas_reward_distributor::*;

use structopt::StructOpt;

use std::collections::HashMap;
use std::path::PathBuf;

/// Rank nodes of an FBAS and allocate rewards to them accordingly
#[derive(Debug, StructOpt)]
struct Cli {
    /// Path to JSON file describing the FBAS in stellarbeat.org "nodes" format.
    /// Will use STDIN if omitted.
    nodes_path: Option<PathBuf>,

    /// Prior to any analysis, filter out all nodes marked as `"active" == false` in the input
    /// nodes JSON (the one at `nodes_path`).
    #[structopt(short = "i", long = "ignore-inactive-nodes")]
    ignore_inactive_nodes: bool,

    /// Amount to be shared among the nodes.
    #[structopt(short = "r", long = "reward")]
    total_reward: f64,

    /// Ranking algorithm to use.
    #[structopt(short = "a", long = "algorithm")]
    alg: RankingAlg,

    /// Only compute node rankings
    #[structopt(long = "rank-only")]
    _rank_only: bool,

    /// Identify nodes by their pretty name their public key. default is to use node IDs corresponding
    /// to indices in the input file.
    #[structopt(short = "p", long = "pretty")]
    pks: bool,
}

// TODO: NodeIDs -> PKs
fn main() {
    let args = Cli::from_args();
    let ranking_alg = args.alg;
    let total_reward = args.total_reward;
    let fbas = load_fbas(args.nodes_path.as_ref(), args.ignore_inactive_nodes);
    let node_ids: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    if total_reward > 0.0 {
        println!("Reward value = {}", total_reward);
        let allocation = distribute_rewards(ranking_alg, &node_ids, &fbas, args.total_reward);
        println!("Allocation {:?}", allocation);
    } else {
        eprintln!("Reward must be greater than 0!");
        return;
    }
    if args.pks {
        let _public_keys = to_public_keys(node_ids, &fbas);
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
fn compute_influence(fbas: &Fbas, alg: RankingAlg) -> Vec<Score> {
    rank_nodes(fbas, alg)
}

/// Distribute the reward between nodes based on their contribution as calculated by a ranking
/// algorithm
fn distribute_rewards(
    algo: RankingAlg,
    nodes: &[NodeId],
    fbas: &Fbas,
    reward_value: f64,
) -> HashMap<NodeId, (f64, Score)> {
    match algo {
        RankingAlg::NodeRank => graph_theory_distribution(nodes, fbas, reward_value),
        RankingAlg::PowerIndex => game_theory_distribution(fbas, reward_value),
    }
}
