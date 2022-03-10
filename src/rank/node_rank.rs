use crate::*;

use fbas_analyzer::{Fbas, NodeId, QuorumSet};
use std::collections::{HashMap, HashSet};

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

/// Given a node ID, returns the NodeRank score of the node
/// all_quorum_sets_containing_node: List of quorum sets that contain node_id
/// sets_to_generators: Map of quorum set hashes and a set of nodes that creates them
/// pr_scores: All FBAS' nodes PR scores
fn compute_node_rank(
    node_id: NodeId,
    qsets_containting_node: Option<&HashSet<QuorumSet>>,
    sets_to_generators: &HashMap<String, HashSet<NodeId>>,
    pr_scores: &[Score],
) -> Score {
    let mut node_rank: Score = Score::default();
    match qsets_containting_node {
        Some(involving_sets) => {
            for set in involving_sets {
                let creators = get_list_of_creators_for_quorum_set(set, sets_to_generators);
                let pr_sum: Score = creators.iter().map(|&v| pr_scores[v] as Score).sum();
                let quorum_set_weight = node_weight_in_quorum_set(node_id, set);
                node_rank += pr_sum * quorum_set_weight;
            }
        }
        None => {
            eprintln!("Node {} not in quorum sets..", node_id);
        }
    }
    round_to_three_places(node_rank)
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
        let pr_scores = fbas.rank_nodes();
        let node_weight = 0.666; // calculated manually
        let pr_sum: Score = pr_scores.iter().map(|&v| v as f64).sum();
        let actual = compute_node_rank_for_fbas(&all_nodes, &fbas);
        let expected = vec![
            pr_sum * node_weight,
            pr_sum * node_weight,
            pr_sum * node_weight,
        ];
        assert_eq!(actual, expected);
    }
    #[test]
    // test case: same quorum set is created by two nodes with PR scores 0.01 and 0.02
    fn node_rank_from_paper_example() {
        let mut fbas = Fbas::new();
        fbas.add_generic_node(QuorumSet::new_empty());
        let mut quorum_set = QuorumSet::new(vec![0, 1], vec![], 3);
        quorum_set.inner_quorum_sets = vec![QuorumSet::new(vec![2, 3], vec![], 1)];
        fbas.add_generic_node(QuorumSet::new_empty());
        let node_two = fbas.add_generic_node(quorum_set.clone());
        let _ = fbas.add_generic_node(quorum_set);

        let qsets_to_nodes = map_quorum_sets_to_generators(&fbas);
        let sets_containing_node = all_quorum_sets_containing_node(node_two, &fbas);
        let pr_scores = [0.0, 0.0, 0.02, 0.01];

        let actual = compute_node_rank(
            node_two,
            Some(&sets_containing_node),
            &qsets_to_nodes,
            &pr_scores,
        );
        let expected = 0.011; // calculated by self
        assert_eq!(expected, actual);
    }
}
