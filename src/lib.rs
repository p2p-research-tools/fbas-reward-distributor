pub mod dist;
pub mod rank;
pub mod report;
pub mod stats;
pub mod types;

pub use dist::*;
pub use rank::*;
pub use report::*;
pub use stats::*;
pub use types::*;
pub type Score = f64;
pub type Reward = f64;

use fbas_analyzer::NodeId;

pub type Coalition = fbas_analyzer::NodeIdSet;

/// Algorithm to use when ranking nodes
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RankingAlg {
    /// An extension of PageRank. See the function 'rank_nodes_using_node_rank' for more
    NodeRank,
    PowerIndexEnum(Option<Vec<NodeId>>),
    /// Expects the number of samples to use
    PowerIndexApprox(usize),
}
