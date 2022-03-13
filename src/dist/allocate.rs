use crate::*;
use fbas_analyzer::Fbas;

use fbas_analyzer::NodeId;
use std::collections::HashMap;

/// Distribute rewards according to NodeRank scores and return a map of NodeId, score, reward
pub fn graph_theory_distribution(
    nodes: &[NodeId],
    fbas: &Fbas,
    reward: f64,
) -> HashMap<NodeId, (Score, f64)> {
    let mut rewards = HashMap::default();
    let scores = compute_node_rank_for_fbas(nodes, fbas);
    let node_rank_sum: Score = scores.iter().map(|&v| v as f64).sum();
    for (node, node_score) in scores.iter().enumerate() {
        // normalise values nr/sum(nr)
        let reward_factor = node_score / node_rank_sum;
        let reward = reward_factor * reward;
        rewards.insert(node, (scores[node], reward));
    }
    rewards
}

/// Distribute rewards proportionally to SS power index and return a map of NodeId, score, reward
pub fn exact_game_theory_distribution(fbas: &Fbas, reward: f64) -> HashMap<NodeId, (Score, f64)> {
    let game = new_game_from_fbas(fbas);
    let scores = game.compute_exact_ss_power_index_for_game();
    allocate_reward_to_players(scores, reward)
}

/// Distribute rewards proportionally to SS power index and return a map of NodeId, score, reward
pub fn approx_game_theory_distribution(fbas: &Fbas, reward: f64) -> HashMap<NodeId, (Score, f64)> {
    let game = new_game_from_fbas(fbas);
    let scores = game.compute_approx_ss_power_index_for_game();
    allocate_reward_to_players(scores, reward)
}

fn new_game_from_fbas(fbas: &Fbas) -> CooperativeGame {
    let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    CooperativeGame::init_from_fbas(&all_nodes, fbas)
}

fn allocate_reward_to_players(scores: Vec<Score>, reward: f64) -> HashMap<usize, (Score, f64)> {
    let mut rewards = HashMap::default();
    for (node, node_score) in scores.iter().enumerate() {
        let share = node_score * reward;
        rewards.insert(node, (scores[node], share));
    }
    rewards
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use fbas_analyzer::Fbas;
    use std::path::Path;

    #[test]
    fn allocate_rewards_simple_fbas_noderank() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let reward = 1.0;
        let noderanks = compute_node_rank_for_fbas(&all_nodes, &fbas);
        let actual = graph_theory_distribution(&all_nodes, &fbas, reward);
        let expected = HashMap::from([
            (0, (noderanks[0], reward / 3.0)),
            (1, (noderanks[1], reward / 3.0)),
            (2, (noderanks[1], reward / 3.0)),
        ]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn allocate_rewards_simple_fbas_exact_powerindex() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let reward = 1.0;
        let actual = exact_game_theory_distribution(&fbas, reward);
        let expected = HashMap::from([
            (0, (1.0 / 3.0, reward / 3.0)),
            (1, (1.0 / 3.0, reward / 3.0)),
            (2, (1.0 / 3.0, reward / 3.0)),
        ]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn allocate_rewards_simple_fbas_approx_powerindex() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let reward = 10.0;
        let actual_rewards = approx_game_theory_distribution(&fbas, reward);
        let expected_rewards = HashMap::from([
            (0, (1.0 / 3.0, reward / 3.0)),
            (1, (1.0 / 3.0, reward / 3.0)),
            (2, (1.0 / 3.0, reward / 3.0)),
        ]);
        for (i, expected) in expected_rewards.into_iter().enumerate() {
            let actual = actual_rewards.get(&i).unwrap();
            assert_relative_eq!(expected.1 .1, actual.1);
        }
    }
}
