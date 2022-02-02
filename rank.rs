use super::*;

use std::collections::HashMap;

/// Algorithm to use when ranking nodes
#[derive(Debug, PartialEq, Eq)]
pub enum RankingAlg {
    /// An adaptation of Google's PageRank
    PageRank,
    /// An extension of PageRank. See the function 'rank_nodes_using_node_rank' for more
    NodeRank,
}

impl Fbas {
    pub fn rank_nodes(&self, ranking_algo: RankingAlg) -> Vec<RankScore> {
        let all_nodes: Vec<NodeId> = (0..self.nodes.len()).collect();
        if ranking_algo == RankingAlg::PageRank {
            rank_nodes_using_page_rank(&all_nodes, self)
        } else {
            rank_nodes_using_node_rank(&all_nodes, self)
        }
    }
}

/// Rank nodes using an adaptation of the page rank algorithm (no dampening, fixed number of runs,
/// no distinction between validators and inner quorum set validators). Links from nodes not in
/// `nodes` are ignored.
// TODO dedup / harmonize this with Graph::get_rank_scores
pub fn rank_nodes_using_page_rank(nodes: &[NodeId], fbas: &Fbas) -> Vec<RankScore> {
    let nodes_set: NodeIdSet = nodes.iter().cloned().collect();
    assert_eq!(nodes.len(), nodes_set.len());

    let runs = 100;
    let starting_score = 1. / nodes.len() as RankScore;

    let mut scores: Vec<RankScore> = vec![starting_score; fbas.nodes.len()];
    let mut last_scores: Vec<RankScore>;

    for _ in 0..runs {
        last_scores = scores;
        scores = vec![0.; fbas.nodes.len()];

        for node_id in nodes.iter().copied() {
            let node = &fbas.nodes[node_id];
            let trusted_nodes = node.quorum_set.contained_nodes();
            let l = trusted_nodes.len() as RankScore;

            for trusted_node_id in trusted_nodes
                .into_iter()
                .filter(|&id| nodes_set.contains(id))
            {
                scores[trusted_node_id] += last_scores[node_id] / l;
            }
        }
    }
    debug!(
        "Non-zero ranking scores: {:?}",
        scores
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, s)| s > 0.)
            .collect::<Vec<(usize, RankScore)>>()
    );
    scores
}

/// NodeRank is an extension of PageRank proposed by Kim et al. in the paper 'Is Stellar as Secure
/// As You Think?'.
pub fn rank_nodes_using_node_rank(nodes: &[NodeId], fbas: &Fbas) -> Vec<RankScore> {
    let page_rank_scores = rank_nodes_using_page_rank(nodes, fbas);
    // A map of <NodeID, [qsets node is in]>
    let sets_involving_node: HashMap<NodeId, HashSet<QuorumSet>> = nodes
        .iter()
        .map(|&v| (v, all_quorum_sets_containing_node(v, fbas)))
        .collect();
    let sets_generators_map = map_quorum_sets_to_generators(fbas);
    let nr_scores: Vec<RankScore> = nodes
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

/// Rank nodes and sort them by "highest rank score first"
pub fn sort_by_rank(mut nodes: Vec<NodeId>, fbas: &Fbas, ranking_algo: RankingAlg) -> Vec<NodeId> {
    let scores = if ranking_algo == RankingAlg::PageRank {
        rank_nodes_using_page_rank(&nodes, fbas)
    } else {
        rank_nodes_using_node_rank(&nodes, fbas)
    };

    nodes.sort_by(|x, y| scores[*y].partial_cmp(&scores[*x]).unwrap());
    nodes
}

/// Distribute rewards according to the ranking and return a map of NodeId, score, reward
pub fn compute_reward_distribution(
    scores: &[RankScore],
    reward: f64,
) -> HashMap<NodeId, (RankScore, f64)> {
    let mut rewards = HashMap::default();
    let node_rank_sum: RankScore = scores.iter().map(|&v| v as f64).sum();
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
        let all_nodes: Vec<NodeId> = (0..fbas.nodes.len()).collect();
        let actual = rank_nodes_using_node_rank(&all_nodes, &fbas);
        // PR scores computed using the impl in rank.rs
        let pr_scores = rank_nodes_using_page_rank(&all_nodes, &fbas);
        let node_weight = 0.6666666666666666; // calculated manually
        let pr_sum: RankScore = pr_scores.iter().map(|&v| v as f64).sum();
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
        let all_nodes: Vec<NodeId> = (0..fbas.nodes.len()).collect();
        let reward = 1.0;
        let noderanks = rank_nodes_using_node_rank(&all_nodes, &fbas);
        let actual = compute_reward_distribution(&noderanks, reward);
        let expected = HashMap::from([
            (0, (noderanks[0], reward / 3.0)),
            (1, (noderanks[1], reward / 3.0)),
            (2, (noderanks[1], reward / 3.0)),
        ]);
        assert_eq!(actual, expected);
    }
}
