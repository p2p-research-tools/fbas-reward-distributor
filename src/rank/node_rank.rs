use crate::*;

use fbas_analyzer::{Fbas, NodeId, QuorumSet};
use std::collections::{HashMap, HashSet};

pub fn rank_nodes(fbas: &Fbas, ranking_algo: RankingAlg) -> Vec<Score> {
    let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    if ranking_algo == RankingAlg::PageRank {
        compute_page_rank_for_fbas(fbas)
    } else {
        compute_node_rank_for_fbas(&all_nodes, fbas)
    }
}

pub fn compute_page_rank_for_fbas(fbas: &Fbas) -> Vec<Score> {
    fbas.rank_nodes()
}

/// NodeRank is an extension of PageRank proposed by Kim et al. in the paper 'Is Stellar as Secure
/// As You Think?'.
pub fn compute_node_rank_for_fbas(nodes: &[NodeId], fbas: &Fbas) -> Vec<Score> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn node_rank_for_simple_symmetric_fbas() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        // PR scores computed using the impl in rank.rs
        let pr_scores = compute_page_rank_for_fbas(&fbas);
        let node_weight = 0.6666666666666666; // calculated manually
        let pr_sum: Score = pr_scores.iter().map(|&v| v as f64).sum();
        let actual = compute_node_rank_for_fbas(&all_nodes, &fbas);
        let expected = vec![
            pr_sum * node_weight,
            pr_sum * node_weight,
            pr_sum * node_weight,
        ];
        assert_eq!(actual, expected);
    }
}
