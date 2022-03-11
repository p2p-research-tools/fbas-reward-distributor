use crate::*;

use fbas_analyzer::{Fbas, NodeId};

pub fn rank_nodes(fbas: &Fbas, ranking_algo: RankingAlg) -> Vec<Score> {
    let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    if ranking_algo == RankingAlg::PowerIndex {
        CooperativeGame::compute_exact_ss_power_index_for_game(&CooperativeGame::init_from_fbas(
            &all_nodes, fbas,
        ))
    } else {
        compute_node_rank_for_fbas(&all_nodes, fbas)
    }
}
