use crate::*;

use fbas_analyzer::{Fbas, NodeId, QuorumSet};
use std::collections::{HashMap, HashSet};

/// Algorithm to use when ranking nodes
#[derive(Debug, PartialEq, Eq)]
pub enum RankingAlg {
    /// An adaptation of Google's PageRank
    PageRank,
    /// An extension of PageRank. See the function 'rank_nodes_using_node_rank' for more
    NodeRank,
}

pub fn rank_nodes(fbas: &Fbas, ranking_algo: RankingAlg) -> Vec<Score> {
    let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    if ranking_algo == RankingAlg::PageRank {
        get_scores_page_rank(fbas)
    } else {
        get_scores_node_rank(&all_nodes, fbas)
    }
}

pub fn get_scores_page_rank(fbas: &Fbas) -> Vec<Score> {
    fbas.rank_nodes()
}

/// NodeRank is an extension of PageRank proposed by Kim et al. in the paper 'Is Stellar as Secure
/// As You Think?'.
pub fn get_scores_node_rank(nodes: &[NodeId], fbas: &Fbas) -> Vec<Score> {
    let page_rank_scores = fbas.rank_nodes();
    // A map of <NodeID, [qsets node is in]>
    let sets_involving_node: HashMap<NodeId, HashSet<QuorumSet>> = nodes
        .iter()
        .map(|&v| (v, all_quorum_sets_containing_node(v, fbas)))
        .collect();
    let sets_generators_map = map_quorum_sets_to_generators(fbas);
    let nr_scores: Vec<Score> = nodes
        .iter()
        .map(|&v| {
            compute_node_rank(
                v,
                sets_involving_node.get(&v),
                &sets_generators_map,
                &page_rank_scores,
            )
        })
        .collect();
    nr_scores
}

/// Distribute rewards according to the ranking and return a map of NodeId, score, reward
pub fn compute_reward_distribution(scores: &[Score], reward: f64) -> HashMap<NodeId, (Score, f64)> {
    let mut rewards = HashMap::default();
    let node_rank_sum: Score = scores.iter().map(|&v| v as f64).sum();
    for (node, node_score) in scores.iter().enumerate() {
        // normalise values nr/sum(nr)
        let reward_factor = node_score / node_rank_sum;
        let reward = reward_factor * reward;
        rewards.insert(node, (scores[node], reward));
    }
    rewards
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn node_rank_for_simple_symmetric_fbas() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        // PR scores computed using the impl in rank.rs
        let pr_scores = get_scores_page_rank(&fbas);
        let node_weight = 0.6666666666666666; // calculated manually
        let pr_sum: Score = pr_scores.iter().map(|&v| v as f64).sum();
        let actual = get_scores_node_rank(&all_nodes, &fbas);
        let expected = vec![
            pr_sum * node_weight,
            pr_sum * node_weight,
            pr_sum * node_weight,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn allocate_rewards_simple_fbas() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let reward = 1.0;
        let noderanks = get_scores_node_rank(&all_nodes, &fbas);
        let actual = compute_reward_distribution(&noderanks, reward);
        let expected = HashMap::from([
            (0, (noderanks[0], reward / 3.0)),
            (1, (noderanks[1], reward / 3.0)),
            (2, (noderanks[1], reward / 3.0)),
        ]);
        assert_eq!(actual, expected);
    }
}
