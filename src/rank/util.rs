use fbas_analyzer::*;
use rug::Integer;
use sha3::{Digest, Sha3_256};
use std::collections::{HashMap, HashSet};

/// Iterates through all quorum sets and
/// Returns a map of quorum set hashes and a list of nodes that created that quorum set
pub(crate) fn map_quorum_sets_to_generators(fbas: &Fbas) -> HashMap<String, HashSet<NodeId>> {
    let mut generators: HashMap<String, HashSet<NodeId>> = HashMap::default();
    let nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
    for v in nodes.iter() {
        let quorum_set = if let Some(qset) = fbas.get_quorum_set(*v) {
            qset
        } else {
            QuorumSet::new_empty()
        };
        let quorum_set_hash = hex::encode(Sha3_256::digest(quorum_set.into_id_string().as_bytes()));
        if let Some(hash) = generators.get_mut(&quorum_set_hash) {
            hash.insert(*v);
        } else {
            generators.insert(quorum_set_hash, HashSet::from([*v]));
        };
    }
    generators
}

/// Returns all quorum sets in the FBAS in which the node is included in the outer quorum set
pub(crate) fn all_quorum_sets_containing_node(node_id: NodeId, fbas: &Fbas) -> HashSet<QuorumSet> {
    let mut qsets_containting_node: HashSet<QuorumSet> = HashSet::default();
    for v in fbas.all_nodes().iter() {
        let quorum_set = if let Some(qset) = fbas.get_quorum_set(v) {
            qset
        } else {
            QuorumSet::new_empty()
        };
        if quorum_set.contained_nodes().contains(node_id) {
            qsets_containting_node.insert(quorum_set.clone());
        }
    }
    qsets_containting_node
}

// T/|Q|
fn qset_weight(quorum_set: &QuorumSet) -> f64 {
    quorum_set.threshold as f64 / quorum_set.contained_nodes().len() as f64
}

// funky a_k-1(Q, v) formula and implementation
pub(crate) fn node_weight_in_quorum_set(node_id: NodeId, quorum_set: &QuorumSet) -> f64 {
    let mut weight = 1.0;
    let nesting_depth = nodes_nesting_depth(quorum_set, node_id);
    match nesting_depth {
        // Base case: not found in qset
        0 => {
            weight *= 1.0;
            weight
        }
        _ => {
            weight *= qset_weight(quorum_set);
            // should actually always take the next nested set..
            weight *= node_weight_in_quorum_set(
                node_id,
                &find_next_quorum_set_containing_node(quorum_set, node_id),
            );
            weight
        }
    }
}

/// Returns the first (inner) quorum set found that the node is included in
fn find_next_quorum_set_containing_node(quorum_set: &QuorumSet, node_id: NodeId) -> QuorumSet {
    for set in &quorum_set.inner_quorum_sets {
        if set.contained_nodes().contains(node_id) {
            return set.clone();
        }
    }
    QuorumSet::new_empty()
}

/// Counting starts at 1 and 0 means the node was not found in the quorum set.
/// If a node is in multiple sets, its first level is returned
fn nodes_nesting_depth(quorum_set: &QuorumSet, node: NodeId) -> usize {
    let mut level = 0;
    if is_in_qset(&quorum_set.validators, node) {
        level += 1;
    } else {
        // if a node is in the xth inner set of this inner qset, it means its in x+1st level in the whole quorum set
        for inner in quorum_set.inner_quorum_sets.iter() {
            // check before incrementing in case node wasn't found
            let depth = depth_in_inner_sets(inner, node);
            if depth != 0 {
                level += depth + 1;
                break;
            }
        }
    }
    level
}

fn is_in_qset(validators: &[NodeId], node: NodeId) -> bool {
    validators.iter().any(|&validator| validator == node)
}

fn depth_in_inner_sets(inner_quorum_set: &QuorumSet, node: NodeId) -> usize {
    let mut depth = 0;
    // 1 means it was found in the validators set, 0 wasn't found
    if is_in_qset(&inner_quorum_set.validators, node) {
        depth += 1;
        return depth;
    } else {
        depth += 1;
        for (idx, inner) in inner_quorum_set.inner_quorum_sets.iter().enumerate() {
            if is_in_qset(&inner.validators, node) {
                // idx + 1 because the counter starts at 0
                // add depth to that to get the level in this quorum set
                depth += idx + 1;
                break;
            }
        }
    }
    depth
}

/// Gets a map of quorum set hashes and node IDs returns the nodes that create the exact quorum set
pub(crate) fn get_list_of_creators_for_quorum_set(
    quorum_set: &QuorumSet,
    sets_to_nodes: &HashMap<String, HashSet<NodeId>>,
) -> HashSet<NodeId> {
    let qset_hash = hex::encode(Sha3_256::digest(
        quorum_set.clone().into_id_string().as_bytes(),
    ));
    let creators = if let Some(same_hash) = sets_to_nodes.get(&qset_hash) {
        same_hash.clone()
    } else {
        HashSet::default()
    };
    creators
}

pub(crate) fn n_factorial(n: usize) -> Integer {
    let n = n as u128;
    if n == 0 {
        return Integer::from(1);
    }
    let mut factorial = Integer::from(1);
    for i in 2..n {
        factorial *= i;
    }
    factorial * n
}

pub(crate) fn round_to_three_places(n: f64) -> f64 {
    f64::trunc(n * 1000.0) / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn flat_qset(validators: &[NodeId], threshold: usize) -> QuorumSet {
        QuorumSet {
            threshold,
            validators: validators.iter().copied().collect(),
            inner_quorum_sets: vec![],
        }
    }
    #[test]
    fn level_of_nesting_in_top_level_quorum_set() {
        let mut quorum_set = flat_qset(&[0, 1], 3);
        quorum_set.inner_quorum_sets = vec![flat_qset(&[2, 3, 4], 2), flat_qset(&[4, 5, 6], 2)];
        let actual = nodes_nesting_depth(&quorum_set, 0);
        let expected = 1;
        assert_eq!(expected, actual);
    }
    #[test]
    fn level_of_nesting_in_inner_qourum_set() {
        let mut quorum_set = flat_qset(&[0, 1], 3);
        quorum_set.inner_quorum_sets = vec![flat_qset(&[2, 3, 4], 2), flat_qset(&[4, 5, 6], 2)];
        let actual = depth_in_inner_sets(&quorum_set.inner_quorum_sets[0], 3);
        let expected = 1;
        assert_eq!(expected, actual);
        quorum_set.inner_quorum_sets[1].inner_quorum_sets = vec![flat_qset(&[7, 8], 2)];
        let actual = depth_in_inner_sets(&quorum_set.inner_quorum_sets[1], 7);
        let expected = 2;
        assert_eq!(expected, actual);
    }
    #[test]
    fn node_nested_in_two_inner_sets() {
        let mut quorum_set = flat_qset(&[0, 1], 3);
        quorum_set.inner_quorum_sets = vec![flat_qset(&[2, 3, 4], 2), flat_qset(&[4, 5, 6], 2)];
        let actual = nodes_nesting_depth(&quorum_set, 4);
        let expected = 2;
        assert_eq!(expected, actual);
    }
    #[test]
    fn node_nested_beyond_second_inner_set() {
        let mut quorum_set = flat_qset(&[0, 1], 3);
        quorum_set.inner_quorum_sets = vec![flat_qset(&[2, 3], 2), flat_qset(&[1, 3], 2)];
        quorum_set.inner_quorum_sets[0].inner_quorum_sets = vec![flat_qset(&[4, 5], 2)];
        let actual = nodes_nesting_depth(&quorum_set, 4);
        let expected = 3;
        assert_eq!(expected, actual);
    }
    #[test]
    fn contains_all_qsets_with_node() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));

        let node_id = 0;
        let actual = all_quorum_sets_containing_node(node_id, &fbas);
        let expected = HashSet::from([
            flat_qset(&[0, 1, 2], 2),
            flat_qset(&[0, 1, 2], 2),
            flat_qset(&[0, 1, 2], 2),
        ]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn contained_in_sets_wont_panic_if_node_is_not_in_qsets() {
        let mut fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        fbas.add_generic_node(QuorumSet::new_empty());
        let node_id = 4;
        let actual = all_quorum_sets_containing_node(node_id, &fbas);
        let expected = HashSet::from([]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn find_node_in_quorum_set() {
        let mut quorum_set = flat_qset(&[0, 1], 3);
        quorum_set.inner_quorum_sets = vec![flat_qset(&[2, 3, 4], 2), flat_qset(&[4, 5, 6], 2)];
        let actual = find_next_quorum_set_containing_node(&quorum_set, 4);
        let expected = flat_qset(&[2, 3, 4], 2);
        assert_eq!(expected, actual);
    }
    #[test]
    fn node_weight_in_quorum_set_paper_example() {
        let mut quorum_set = flat_qset(&[0, 1], 3);
        quorum_set.inner_quorum_sets = vec![flat_qset(&[2, 3], 1)];
        let actual = node_weight_in_quorum_set(2, &quorum_set);
        let expected = 0.375; // calculated by self
        assert_eq!(expected, actual);
    }
    #[test]
    fn correct_generators_to_qset_map() {
        let mut fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        fbas.add_generic_node(QuorumSet::new_empty());
        let actual = map_quorum_sets_to_generators(&fbas);
        let expected = HashMap::from([
            (
                String::from("0f93959de22e7a5c4461e08879d090f23668b0def8b22287ed819d8fc946ac0f"),
                HashSet::from([0, 1, 2]),
            ),
            (
                String::from("adb4a6e5d29e47a22efd25786bdc0f7d457b7d100868a347dc3c301f3b67d7fc"),
                HashSet::from([3]),
            ),
        ]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn list_of_generators_for_quorum_set() {
        let mut fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        fbas.add_generic_node(QuorumSet::new_empty());
        let sets_generators_map = map_quorum_sets_to_generators(&fbas);
        let actual = get_list_of_creators_for_quorum_set(
            &fbas.get_quorum_set(0).unwrap(),
            &sets_generators_map,
        );
        let expected = HashSet::from([0, 1, 2]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn factorial() {
        let numbers = vec![0, 1, 3];
        let expected = vec![1, 1, 6];
        for (i, n) in numbers.iter().enumerate() {
            let actual = n_factorial(*n);
            assert_eq!(expected[i], actual);
        }
    }
    #[test]
    fn round() {
        let pi = 3.1415926535897932384626433832;
        let actual = round_to_three_places(pi);
        let expected = 3.141;
        assert_eq!(actual, expected);
    }
}
