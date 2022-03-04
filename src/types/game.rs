use crate::Coalition;
use fbas_analyzer::{Fbas, NodeId};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CooperativeGame<'a> {
    /// The FBAS
    pub(crate) fbas: &'a Fbas,
    /// The set of players
    pub(crate) players: Vec<usize>,
    /// The coalitions for which a player is critical
    pub(crate) players_critical_coalitions: HashMap<usize, Vec<Coalition>>,
}

impl<'a> CooperativeGame<'a> {
    /// Sets the number of players and corresponding FBAS
    pub fn init_from_fbas(nodes: &[NodeId], fbas: &'a Fbas) -> Self {
        let mut players: Vec<usize> = nodes.into();
        players.dedup();
        let game = Self {
            fbas,
            players,
            players_critical_coalitions: HashMap::default(),
        };
        let winning_coalitions = game.find_winning_coalitions(nodes.len());
        let players_critical_coalitions = game
            .players
            .iter()
            .map(|v| (*v, Self::player_is_critical(*v, &winning_coalitions)))
            .collect();
        Self {
            players_critical_coalitions,
            ..game
        }
    }

    pub(crate) fn coalitions_cardinatily(coalition: &Coalition) -> usize {
        coalition.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fbas_analyzer::bitset;
    use std::path::Path;

    #[test]
    fn from_fbas_to_game() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let all_nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let expected = CooperativeGame {
            fbas: &fbas,
            players: vec![0, 1, 2],
            players_critical_coalitions: HashMap::from([
                (0, vec![bitset![0, 1], bitset![0, 2]]),
                (1, vec![bitset![0, 1], bitset![1, 2]]),
                (2, vec![bitset![0, 2], bitset![1, 2]]),
            ]),
        };
        let actual = CooperativeGame::init_from_fbas(&all_nodes, &fbas);
        assert_eq!(expected.players, actual.players);
        assert_eq!(
            expected.players_critical_coalitions.len(),
            actual.players_critical_coalitions.len()
        );
        for set in expected.players_critical_coalitions.keys() {
            assert!(actual.players_critical_coalitions.get(&set).is_some());
        }
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
    fn players_critical_sets_equals_nodes_quorums() {
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
            vec![bitset![0, 1, 2], bitset![0, 3, 4], bitset![0, 1, 2, 3, 4]],
            vec![bitset![0, 1, 2], bitset![0, 1, 2, 3, 4]],
            vec![bitset![0, 1, 2], bitset![0, 1, 2, 3, 4]],
            vec![bitset![0, 3, 4], bitset![0, 1, 2, 3, 4]],
            vec![bitset![0, 3, 4], bitset![0, 1, 2, 3, 4]],
        ];
        for player in game.players {
            let actual = game.players_critical_coalitions.get(&player).unwrap();
            for c in &expected[player] {
                assert!(actual.contains(&c));
            }
        }
    }
}
