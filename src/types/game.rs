use crate::Coalition;
use fbas_analyzer::{Fbas, NodeId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CooperativeGame<'a> {
    /// The FBAS
    pub(crate) fbas: &'a Fbas,
    /// The set of players
    pub(crate) players: Vec<NodeId>,
    /// The top tier of the FBAS. Relevant for the approximation only
    pub(crate) top_tier: Option<Vec<NodeId>>,
}

impl<'a> CooperativeGame<'a> {
    /// Sets the number of players and corresponding FBAS
    pub fn init_from_fbas(nodes: &[NodeId], fbas: &'a Fbas) -> Self {
        let mut players: Vec<NodeId> = nodes.into();
        players.dedup();
        Self {
            fbas,
            players,
            top_tier: None,
        }
    }

    /// Sets the number of players and corresponding FBAS
    pub fn init_from_fbas_with_top_tier(
        all_nodes: &[NodeId],
        top_tier: &[NodeId],
        fbas: &'a Fbas,
    ) -> Self {
        let mut players: Vec<NodeId> = all_nodes.into();
        players.dedup();
        Self {
            fbas,
            players,
            top_tier: Some(top_tier.into()),
        }
    }

    pub(crate) fn coalitions_cardinatily(coalition: &Coalition) -> usize {
        coalition.len()
    }

    pub(crate) fn get_involved_nodes(fbas: &Fbas) -> Vec<NodeId> {
        let min_quorums = fbas_analyzer::find_minimal_quorums(fbas);
        fbas_analyzer::involved_nodes(&min_quorums)
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fbas_analyzer::bitset;
    use std::path::Path;

    #[test]
    fn mqs_contained_nodes() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let expected = vec![0, 1, 2];
        let actual = CooperativeGame::get_involved_nodes(&fbas);
        assert_eq!(expected, actual);
    }

    #[test]
    fn from_fbas_to_game() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let expected = CooperativeGame {
            fbas: &fbas,
            players: vec![0, 1, 2],
            top_tier: None,
        };
        let actual = CooperativeGame::init_from_fbas(&all_nodes, &fbas);
        assert_eq!(expected.players, actual.players);
    }

    #[test]
    fn set_cardinality() {
        let coalitions = vec![bitset![], bitset![0, 4, 8]];
        let expected = vec![0, 3];
        for (i, c) in coalitions.iter().enumerate() {
            assert_eq!(expected[i], CooperativeGame::coalitions_cardinatily(c));
        }
    }
    #[test]
    fn players_critical_sets_equals_sets_with_quorums() {
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
        // U = {0, 1, 2}, {0, 3, 4}, {0, 1, 2, 3, 4}
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let game = CooperativeGame::init_from_fbas(&all_nodes, &fbas);
        let expected = vec![
            vec![
                bitset![0, 1, 2],
                bitset![0, 3, 4],
                bitset![0, 1, 2, 3],
                bitset![0, 1, 2, 4],
                bitset![0, 2, 3, 4],
                bitset![0, 1, 3, 4],
                bitset![0, 1, 2, 3, 4],
            ],
            vec![bitset![0, 1, 2], bitset![0, 1, 2, 3], bitset![0, 1, 2, 4]],
            vec![bitset![0, 1, 2], bitset![0, 1, 2, 3], bitset![0, 1, 2, 4]],
            vec![bitset![0, 3, 4], bitset![0, 2, 3, 4], bitset![0, 1, 3, 4]],
            vec![bitset![0, 3, 4], bitset![0, 2, 3, 4], bitset![0, 1, 3, 4]],
        ];
        let top_tier = CooperativeGame::get_involved_nodes(&fbas);
        let winning = game.find_winning_coalitions(&top_tier);
        let actual: Vec<Vec<Coalition>> = game
            .players
            .iter()
            .map(|p| CooperativeGame::player_is_critical(*p, &winning))
            .collect();
        for i in 0..actual.len() {
            assert!(actual[i].iter().all(|set| expected[i].contains(set)));
        }
    }

    #[test]
    fn init_game_with_tt() {
        let fbas = Fbas::from_json_file(Path::new("test_data/trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let tt = CooperativeGame::get_involved_nodes(&fbas);
        let expected = CooperativeGame {
            fbas: &fbas,
            players: vec![0, 1, 2],
            top_tier: Some(tt.clone()),
        };
        let actual = CooperativeGame::init_from_fbas_with_top_tier(&all_nodes, &tt, &fbas);
        assert_eq!(expected, actual);
    }
}
