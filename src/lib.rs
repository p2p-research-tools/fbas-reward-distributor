pub mod dist;
pub mod rank;
pub mod types;

pub use dist::*;
pub use rank::*;
pub(crate) use types::*;
pub type Score = f64;

pub type Coalition = fbas_analyzer::NodeIdSet;

use std::str::FromStr;

/// Algorithm to use when ranking nodes
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RankingAlg {
    /// An extension of PageRank. See the function 'rank_nodes_using_node_rank' for more
    NodeRank,
    PowerIndex,
}

impl FromStr for RankingAlg {
    type Err = &'static str;
    fn from_str(alg: &str) -> Result<Self, Self::Err> {
        match alg.to_lowercase().as_ref() {
            "noderank" => Ok(RankingAlg::NodeRank),
            "powerindex" => Ok(RankingAlg::PowerIndex),
            _ => Err("Unknown algorithm"),
        }
    }
}
