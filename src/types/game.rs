use crate::Coalition;
use fbas_analyzer::Fbas;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CooperativeGame<'a> {
    /// The FBAS
    pub(crate) fbas: &'a Fbas,
    /// The set of players
    pub(crate) players: HashSet<usize>,
    /// The simple game's characteristic function
    // /// Vec<Coalition> represents 2^N
    //pub(crate) winning_coalitions: Vec<Coalition>,
    /// The coalitions for which a player is critical
    pub(crate) players_critical_coalitions: HashMap<usize, Vec<Coalition>>,
}

impl<'a> CooperativeGame<'a> {
    /// Sets the number of players and corresponding FBAS
    pub fn init_from_fbas(fbas: &'a Fbas) -> Self {
        let nodes = fbas.all_nodes();
        let game = Self {
            fbas,
            players: nodes.iter().collect(),
            players_critical_coalitions: HashMap::default(),
        };
        let winning_coalitions = game.find_winning_coalitions(nodes.len());
        let players_critical_coalitions = nodes
            .iter()
            .map(|v| (v, Self::player_is_critical(v, &winning_coalitions)))
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
        let expected = CooperativeGame {
            fbas: &fbas,
            players: HashSet::from([0, 1, 2]),
            players_critical_coalitions: HashMap::from([
                (0, vec![bitset![0, 1], bitset![0, 2]]),
                (1, vec![bitset![0, 1], bitset![1, 2]]),
                (2, vec![bitset![0, 2], bitset![1, 2]]),
            ]),
        };
        let actual = CooperativeGame::init_from_fbas(&fbas);
        assert_eq!(expected.players.len(), actual.players.len());
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
}
