use crate::*;

use fbas_analyzer::{Fbas, NodeId};

pub fn rank_nodes(fbas: &Fbas, ranking_algo: RankingAlg) -> Vec<Score> {
    let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    match ranking_algo {
        RankingAlg::ExactPowerIndex => CooperativeGame::compute_exact_ss_power_index_for_game(
            &CooperativeGame::init_from_fbas(&all_nodes, fbas),
        ),
        RankingAlg::ApproxPowerIndex(samples, top_tier) => {
            if let Some(tt) = top_tier {
                CooperativeGame::compute_approx_ss_power_index_for_game(
                    &CooperativeGame::init_from_fbas_with_top_tier(&all_nodes, &tt, fbas),
                    samples,
                )
            } else {
                CooperativeGame::compute_approx_ss_power_index_for_game(
                    &CooperativeGame::init_from_fbas(&all_nodes, fbas),
                    samples,
                )
            }
        }
        RankingAlg::NodeRank => compute_node_rank_for_fbas(&all_nodes, fbas),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use std::path::Path;

    #[test]
    fn rank_nodes_with_noderank() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let actual = rank_nodes(&fbas, RankingAlg::NodeRank);
        let expected = vec![0.666, 0.666, 0.666];
        assert_eq!(expected, actual);
    }

    #[test]
    fn rank_nodes_with_power_index() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let actual = rank_nodes(&fbas, RankingAlg::ExactPowerIndex);
        let expected = vec![0.333, 0.333, 0.333];
        assert_eq!(expected, actual);
    }
    #[test]
    fn rank_nodes_with_approx_index() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let actual = rank_nodes(&fbas, RankingAlg::ApproxPowerIndex(100, None));
        let expected = vec![0.333, 0.333, 0.333];
        for i in 0..expected.len() {
            assert_abs_diff_eq!(expected[i], actual[i], epsilon = 0.2f64);
        }
    }
    #[test]
    fn rank_nodes_with_approx_index_with_toptier() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let top_tier = CooperativeGame::get_involved_nodes(&fbas);
        let actual = rank_nodes(&fbas, RankingAlg::ApproxPowerIndex(100, Some(top_tier)));
        let expected = vec![0.333, 0.333, 0.333];
        for i in 0..expected.len() {
            assert_abs_diff_eq!(expected[i], actual[i], epsilon = 0.2f64);
        }
    }
}
