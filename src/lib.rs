pub mod dist;
pub mod io;
pub mod rank;
pub mod types;

pub use dist::*;
pub use rank::*;
pub(crate) use types::*;
pub type Score = f64;
pub type Reward = f64;
pub use io::*;

pub type Coalition = fbas_analyzer::NodeIdSet;

/// Algorithm to use when ranking nodes
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RankingAlg {
    /// An extension of PageRank. See the function 'rank_nodes_using_node_rank' for more
    NodeRank,
    ExactPowerIndex,
    ApproxPowerIndex(usize),
}
