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

use std::str::FromStr;

/// Algorithm to use when ranking nodes
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RankingAlg {
    /// An extension of PageRank. See the function 'rank_nodes_using_node_rank' for more
    NodeRank,
    ExactPowerIndex,
    ApproxPowerIndex,
}

impl FromStr for RankingAlg {
    type Err = &'static str;
    fn from_str(alg: &str) -> Result<Self, Self::Err> {
        match alg.to_lowercase().as_ref() {
            "noderank" => Ok(RankingAlg::NodeRank),
            "exact-powerindex" => Ok(RankingAlg::ExactPowerIndex),
            "approx-powerindex" => Ok(RankingAlg::ApproxPowerIndex),
            _ => Err("Unknown algorithm"),
        }
    }
}
