pub mod dist;
pub mod rank;
pub mod types;

pub use dist::*;
pub use rank::*;
pub use types::*;

pub type Score = f64;

pub type Coalition = fbas_analyzer::NodeIdSet;

/// Algorithm to use when ranking nodes
#[derive(Debug, PartialEq, Eq)]
pub enum RankingAlg {
    /// An adaptation of Google's PageRank
    PageRank,
    /// An extension of PageRank. See the function 'rank_nodes_using_node_rank' for more
    NodeRank,
}
