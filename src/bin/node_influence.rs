use fbas_analyzer::*;
use fbas_node_influence::*;

use structopt::StructOpt;

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
struct Cli {
    /// Path to JSON file describing the FBAS in stellarbeat.org "nodes" format.
    /// Will use STDIN if omitted.
    nodes_path: Option<PathBuf>,

    /// Prior to any analysis, filter out all nodes marked as `"active" == false` in the input
    /// nodes JSON (the one at `nodes_path`).
    #[structopt(short = "i", long = "ignore-inactive-nodes")]
    ignore_inactive_nodes: bool,

    /// Compute allocation for reward of value.
    #[structopt(short = "d", long = "distribute ")]
    total_reward: Option<f64>,

    /// Identify nodes by their pretty name their public key. default is to use node IDs corresponding
    /// to indices in the input file.
    #[structopt(short = "p", long = "pretty")]
    pks: bool,

    /// Scoring algorithm to use when ranking nodes.
    #[structopt(subcommand)]
    rank: RankingAlgConfig,
}

#[derive(Debug, StructOpt)]
enum RankingAlgConfig {
    /// Use a version of Google's PageRank
    Pagerank,
    /// Use an extension of PR (see implementation for details)
    Noderank,
}

// TODO: NodeIDs -> PKs
fn main() {
    let args = Cli::from_args();
    let fbas = load_fbas(args.nodes_path.as_ref(), args.ignore_inactive_nodes);
    let node_ids: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    let ranks = match args.rank {
        RankingAlgConfig::Pagerank => {
            eprintln!("Using PageRank..");
            compute_influence_w_page_rank(&fbas)
        }
        RankingAlgConfig::Noderank => {
            eprintln!("Using NodeRank..");
            compute_influence_w_node_rank(&node_ids, &fbas)
        }
    };
    if let Some(reward) = args.total_reward {
        println!("Reward value = {}", reward);
        let allocation = distribute_rewards(&ranks, reward);
        println!("Allocation {:?}", allocation);
    };
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

/// Rank nodes using PageRank and return a sorted list of nodes
fn compute_influence_w_page_rank(fbas: &Fbas) -> Vec<Score> {
    compute_page_rank_for_fbas(fbas)
}

/// Rank nodes using NodeRank and return a sorted list of nodes
fn compute_influence_w_node_rank(all_nodes: &[NodeId], fbas: &Fbas) -> Vec<Score> {
    compute_node_rank_for_fbas(all_nodes, fbas)
}

/// Distribute the reward between nodes based on their contribution as calculated by a ranking
/// algorithm
fn distribute_rewards(node_scores: &[Score], reward_value: f64) -> HashMap<NodeId, (f64, Score)> {
    compute_reward_distribution(node_scores, reward_value)
}
