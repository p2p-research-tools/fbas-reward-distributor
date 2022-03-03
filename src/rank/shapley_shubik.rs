use crate::*;
use itertools::Itertools;
use std::collections::HashSet;

impl<'a> CooperativeGame<'a> {
    /// Calculates the Shapley-Shubik Index for the players of the game
    /// Returns a list of scores with 0 = node 0's score
    /// A coalition is winning if it is a quorum in the FBAS, losing otherwise
    /// See C. Ndolo Master's thesis for details
    pub fn compute_ss_power_index_for_game(&self) -> Vec<Score> {
        let num_players = self.players.len();
        let total_factorial = n_factorial(num_players as u128);
        let power_indices: Vec<Score> = self
            .players
            .iter()
            .map(|&p| {
                Self::compute_player_power_index(
                    self.players_critical_coalitions.get(&p),
                    num_players,
                    total_factorial,
                )
            })
            .collect();
        power_indices
    }

    /// winning_coalitions: a player's winning coalitions used to find their power index
    /// num_players: number of players in the game
    /// total_factorial: The factorial of num_players
    fn compute_player_power_index(
        winning_coalitions: Option<&Vec<Coalition>>,
        num_players: usize,
        total_factorial: u128,
    ) -> Score {
        if let Some(critical_coalitions) = winning_coalitions {
            critical_coalitions
                .iter()
                .map(|w| ss_probability_for_one_coalition(w, num_players, total_factorial))
                .sum()
        } else {
            0.0
        }
    }

    /// We construct the power set based on the number of players
    /// If a coalition is a quorum, it is a winning coalition
    pub(crate) fn find_winning_coalitions(&self, num_players: usize) -> HashSet<Coalition> {
        let all_coalitions = (0..num_players).powerset().collect::<Vec<_>>();
        let mut winning: HashSet<Coalition> = HashSet::new();
        for s in all_coalitions {
            let quorum = s.into_iter().collect();
            if self.fbas.is_quorum(&quorum) {
                winning.insert(quorum.clone());
            }
        }

        winning
    }

    /// Get a player's winning coalitions, i.e. the quorums that contain the player and lose quorum
    /// 'status' when the player is removed from the set
    /// Alg: Iterate all winning coalitions w and check player is in w
    /// Yes: Remove player. If w is no a longer a quorum, then player is critical
    /// TODO: Maybe parallel as an optimisation?
    pub(crate) fn player_is_critical(
        player: usize,
        winning_coalitions: &HashSet<Coalition>,
    ) -> Vec<Coalition> {
        let mut is_now_losing: Vec<Coalition> = Vec::new();
        for w in winning_coalitions {
            if w.contains(player) {
                let mut w_without_player = w.clone();
                w_without_player.remove(player);
                // It was a quorum before and now it isn't so player must be critical
                if !winning_coalitions.contains(&w_without_player) {
                    is_now_losing.push(w.clone());
                }
            }
        }
        is_now_losing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fbas_analyzer::{bitset, Fbas};
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn all_winning_sets_in_fbas() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let game = CooperativeGame {
            fbas: &fbas,
            players: fbas.all_nodes().iter().collect(),
            players_critical_coalitions: HashMap::default(),
        };
        let actual = game.find_winning_coalitions(3);
        let expected = HashSet::from([
            bitset![0, 1],
            bitset![0, 2],
            bitset![1, 2],
            bitset![0, 1, 2],
        ]);
        assert_eq!(expected, actual);
    }
    #[test]
    fn critical_sets_for_player() {
        let winning = HashSet::from([
            bitset![0, 1],
            bitset![0, 2],
            bitset![1, 2],
            bitset![0, 1, 2],
        ]);
        let expected = vec![bitset![0, 1], bitset![0, 2]];
        let actual = CooperativeGame::player_is_critical(0, &winning);
        assert_eq!(expected.len(), actual.len());
        for set in expected {
            assert!(actual.contains(&set));
        }
    }

    #[test]
    fn single_players_ss_power_index() {
        let winning = vec![bitset![0, 1], bitset![0, 2]];
        let num_players = 3;
        let factorial = 6;
        let expected = 2.0 / 6.0;
        let actual =
            CooperativeGame::compute_player_power_index(Some(&winning), num_players, factorial);
        assert_eq!(expected, actual);
    }

    #[test]
    fn power_index_for_game() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let game = CooperativeGame::init_from_fbas(&fbas);
        let expected = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        let actual = game.compute_ss_power_index_for_game();
        assert_eq!(expected, actual);
    }
}
