use crate::*;
use fbas_analyzer::{Fbas, NodeId};

/// Distribute rewards according to NodeRank scores and return a list of NodeId, score, reward
pub fn graph_theory_distribution(
    nodes: &[NodeId],
    fbas: &Fbas,
    reward: Reward,
    qi_check: bool,
) -> Vec<(NodeId, Score, Reward)> {
    let mut rewards = Vec::default();
    let scores = compute_node_rank_for_fbas(nodes, fbas, qi_check);
    let node_rank_sum: Score = scores.iter().map(|&v| v as Score).sum();
    for (node, node_score) in scores.iter().enumerate() {
        // normalise values nr/sum(nr)
        let reward_factor = node_score / node_rank_sum;
        let reward = round_to_three_places(reward_factor * reward);
        rewards.push((node, scores[node], reward));
    }
    rewards
}

/// Distribute rewards proportionally to SS power index and return a map of NodeId, score, reward
pub fn exact_game_theory_distribution(
    fbas: &Fbas,
    reward: Reward,
    top_tier: Option<Vec<NodeId>>,
    qi_check: bool,
) -> Vec<(NodeId, Score, Reward)> {
    let game = if let Some(tt) = top_tier {
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        CooperativeGame::init_from_fbas_with_top_tier(&all_nodes, &tt, fbas)
    } else {
        new_game_from_fbas(fbas)
    };
    let scores = game.compute_exact_ss_power_index_for_game(qi_check);
    allocate_reward_to_players(scores, reward)
}

/// Distribute rewards proportionally to SS power index and return a map of NodeId, score, reward
pub fn approx_game_theory_distribution(
    num_samples: usize,
    fbas: &Fbas,
    reward: Reward,
    qi_check: bool,
    seed: u64,
) -> Vec<(NodeId, Score, Reward)> {
    let game = new_game_from_fbas(fbas);
    let scores = game.compute_approx_ss_power_index_for_game(num_samples, qi_check, seed);
    allocate_reward_to_players(scores, reward)
}

fn new_game_from_fbas(fbas: &Fbas) -> CooperativeGame {
    let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    CooperativeGame::init_from_fbas(&all_nodes, fbas)
}

fn allocate_reward_to_players(scores: Vec<Score>, reward: Reward) -> Vec<(NodeId, Score, Reward)> {
    let mut rewards = Vec::default();
    for (node, node_score) in scores.iter().enumerate() {
        let share = round_to_three_places(node_score * reward);
        rewards.push((node, scores[node], share));
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
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let reward = 1.0;
        let qi_check = true;
        let noderanks = compute_node_rank_for_fbas(&all_nodes, &fbas, qi_check);
        let actual = graph_theory_distribution(&all_nodes, &fbas, reward, qi_check);
        let expected = vec![
            (0, noderanks[0], round_to_three_places(reward / 3.0)),
            (1, noderanks[1], round_to_three_places(reward / 3.0)),
            (2, noderanks[1], round_to_three_places(reward / 3.0)),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn allocate_rewards_simple_fbas_exact_powerindex() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let reward = 1.0;
        let qi_check = true;
        let actual = exact_game_theory_distribution(&fbas, reward, None, qi_check);
        let expected = vec![
            (0, 0.333, round_to_three_places(reward / 3.0)),
            (1, 0.333, round_to_three_places(reward / 3.0)),
            (2, 0.333, round_to_three_places(reward / 3.0)),
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn allocate_rewards_simple_fbas_approx_powerindex() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let samples = 100;
        let reward = 10.0;
        let qi_check = true;
        let seed = 1;
        let actual_rewards =
            approx_game_theory_distribution(samples, &fbas, reward, qi_check, seed);
        let expected_rewards = vec![
            (0, 1.0 / 3.0, reward / 3.0),
            (1, 1.0 / 3.0, reward / 3.0),
            (2, 1.0 / 3.0, reward / 3.0),
        ];
        for (i, expected) in expected_rewards.into_iter().enumerate() {
            let actual = actual_rewards[i];
            assert_abs_diff_eq!(expected.1, actual.1, epsilon = 0.2f64);
        }
    }
    #[test]
    fn allocate_rewards_simple_fbas_exact_powerindex_with_toptier() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let reward = 1.0;
        let top_tier = vec![0, 1, 2];
        let qi_check = true;
        let actual = exact_game_theory_distribution(&fbas, reward, Some(top_tier), qi_check);
        let expected = vec![
            (0, 0.333, round_to_three_places(reward / 3.0)),
            (1, 0.333, round_to_three_places(reward / 3.0)),
            (2, 0.333, round_to_three_places(reward / 3.0)),
        ];
        assert_eq!(expected, actual);
    }
}
