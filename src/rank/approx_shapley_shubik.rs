use crate::*;
use bit_set::BitSet;
use fbas_analyzer::{Fbas, NodeId};
use rand::seq::SliceRandom;

impl<'a> CooperativeGame<'a> {
    /// Calculates an approximation of the Shapley-Shubik Index for the players of the game using
    /// a sampling algorithm introduced by [Catro et. al](Polynomial calculation of the Shapley value based on
    /// sampling).
    /// A coalition is winning if it contains a quorum in the FBAS, otherwise losing
    /// See C. Ndolo Master's thesis for details
    pub(crate) fn compute_approx_ss_power_index_for_game(&self, num_samples: usize) -> Vec<Score> {
        let top_tier = Self::get_involved_nodes(self.fbas);
        let sample_permutations = generate_sample_permutations(num_samples, &top_tier);
        let power_indices: Vec<Score> = self
            .players
            .iter()
            .map(|&p| {
                Self::compute_approx_ss_power_index_for_player(p, &sample_permutations, self.fbas)
            })
            .collect();
        power_indices
    }

    /// player: ID of player whose score we are computing
    /// permutation_samples: randomly chosen orders of the grand coalition
    /// The estimate is equal to the sum of player's contribution each colution/samples
    fn compute_approx_ss_power_index_for_player(
        player: usize,
        permutation_samples: &[Vec<usize>],
        fbas: &Fbas,
    ) -> Score {
        let mut estimate = Score::default();
        for sample in permutation_samples {
            let pred = pred_of_player_i(player, sample);
            let contribution = compute_player_i_marginal_contribution(player, &pred, fbas);
            estimate += contribution as f64;
        }
        estimate /= permutation_samples.len() as f64;
        round_to_three_places(estimate)
    }
}

/// Given a permutation O, Pre^i(O) is the set of predecessors of the
/// player i in the order O, i.e. Pre^i(O) = {O(1), . . . , O(k − 1)}, if i = O(k))
fn pred_of_player_i(i: usize, permutation: &[usize]) -> Vec<NodeId> {
    match permutation.iter().position(|&idx| idx == i) {
        Some(idx) => permutation.iter().copied().take(idx).collect(),
        None => Vec::default(),
    }
}

/// Expects the predecessors of player as a permutation
/// Return v(pre union player) - v(pred)
/// 1 when pred is losing but union contains a quorums, 0 otherwise
fn compute_player_i_marginal_contribution(player: usize, pred: &[usize], fbas: &Fbas) -> usize {
    let predecessor: BitSet = pred.iter().copied().collect();
    let mut pred_union_player = predecessor.clone();
    pred_union_player.insert(player);
    if fbas_analyzer::contains_quorum(&pred_union_player, fbas)
        && !fbas_analyzer::contains_quorum(&predecessor, fbas)
    {
        1
    } else {
        0
    }
}

/// We create the grand coalition, and randomly select no_samples permutations of it
/// Done by shuffling the grand coalition no_sample many times
/// Bitset wont work here because of order
fn generate_sample_permutations(no_samples: usize, top_tier: &[NodeId]) -> Vec<Vec<NodeId>> {
    let mut grand_coalition: Vec<usize> = top_tier.into();
    // Complexity 0(n) per shuffle
    let mut rng = rand::thread_rng();
    let mut random_permutations: Vec<Vec<usize>> = Vec::default();
    for _ in 0..no_samples {
        grand_coalition.shuffle(&mut rng);
        random_permutations.push(grand_coalition.clone());
    }
    random_permutations
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use fbas_analyzer::NodeId;
    use std::path::Path;

    #[test]
    fn generate_correct_samples() {
        let tt = vec![];
        let actual = generate_sample_permutations(6, &tt);
        assert!(actual.len() == 6);
    }

    #[test]
    fn permutations_predecessors() {
        let player = 0;
        let permutations = vec![vec![0, 1, 2, 3], vec![3, 2, 1, 0]];
        let expected = vec![vec![], vec![3, 2, 1]];
        for i in 0..permutations.len() {
            let actual = pred_of_player_i(player, &permutations[i]);
            assert_eq!(expected[i], actual);
        }
    }

    #[test]
    fn permutations_worth() {
        // U = {0, 1}, {0, 2}, {1, 2} {0, 1, 2}
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let predecessors = vec![vec![0, 1], vec![2, 1], vec![1, 2, 0], vec![1]];
        let players = [2, 0, 0, 2];
        let expected = [0, 0, 0, 1];
        for i in 0..predecessors.len() {
            let actual =
                compute_player_i_marginal_contribution(players[i], &predecessors[i], &fbas);
            assert_eq!(expected[i], actual);
        }
    }

    #[test]
    fn one_players_estimated_index() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let tt = CooperativeGame::get_involved_nodes(&fbas);
        let samples = generate_sample_permutations(100, &tt);
        let actual = CooperativeGame::compute_approx_ss_power_index_for_player(0, &samples, &fbas);
        let expected = 1.0 / 3.0;
        // a and b equal if |a - b| <= epsilon
        assert_abs_diff_eq!(expected, actual, epsilon = 0.2f64);
    }

    #[test]
    fn players_estimated_index() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let tt = CooperativeGame::get_involved_nodes(&fbas);
        let samples = generate_sample_permutations(100, &tt);
        let actual = CooperativeGame::compute_approx_ss_power_index_for_player(0, &samples, &fbas);
        let expected = 1.0 / 3.0;
        assert_abs_diff_eq!(expected, actual, epsilon = 0.2f64);
    }

    #[test]
    fn approx_power_index_for_symmetric_game() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let game = CooperativeGame::init_from_fbas(&all_nodes, &fbas);
        let samples = 100;
        let expected = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        let actual = game.compute_approx_ss_power_index_for_game(samples);
        for e in 0..expected.len() {
            assert_abs_diff_eq!(expected[e], actual[e], epsilon = 0.2f64);
        }
    }

    #[test]
    // Infamous FBAS example with 5 nodes
    fn approx_power_index_for_game_in_paper() {
        let input = r#"[
            {
                "publicKey": "node0",
                "quorumSet": {
                    "threshold": 3,
                    "validators": [
                        "node0",
                        "node1",
                        "node2",
                        "node3",
                        "node4"
                    ]
                }
            },
            {
                "publicKey": "node1",
                "quorumSet": {
                    "threshold": 3,
                    "validators": [
                        "node0",
                        "node1",
                        "node2"
                    ]
                }
            },
            {
                "publicKey": "node2",
                "quorumSet": {
                    "threshold": 3,
                    "validators": [
                        "node0",
                        "node1",
                        "node2"
                    ]
                }
            },
            {
                "publicKey": "node3",
                "quorumSet": {
                    "threshold": 3,
                    "validators": [
                        "node0",
                        "node3",
                        "node4"
                    ]
                }
            },
            {
                "publicKey": "node4",
                "quorumSet": {
                    "threshold": 3,
                    "validators": [
                        "node0",
                        "node3",
                        "node4"
                    ]
                }
            }]"#;
        let fbas = Fbas::from_json_str(&input);
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let game = CooperativeGame::init_from_fbas(&all_nodes, &fbas);
        let samples = 100;
        let expected = vec![7.0 / 15.0, 4.0 / 30.0, 4.0 / 30.0, 4.0 / 30.0, 4.0 / 30.0];
        let actual = game.compute_approx_ss_power_index_for_game(samples);
        for (i, _) in expected.iter().enumerate() {
            assert_abs_diff_eq!(expected[i], actual[i], epsilon = 0.2f64);
        }
    }
}
