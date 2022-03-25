mod game;
mod io;

pub(crate) use game::*;
pub use io::*;

use crate::{Reward, Score};
use fbas_analyzer::NodeId;

pub type NodeRanking = (NodeId, PublicKey, Score);
pub type NodeReward = (NodeId, PublicKey, Score, Reward);
pub type PublicKey = String;
