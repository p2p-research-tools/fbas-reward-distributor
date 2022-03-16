use crate::io::*;
use fbas_analyzer::{to_public_keys, Fbas};

/// Returns a list of NodeRankings sorted by scores
pub fn create_node_ranking_report(
    nodes: &[NodeId],
    scores: Vec<Score>,
    fbas: &Fbas,
    with_pks: bool,
) -> Vec<NodeRanking> {
    let pks = if with_pks {
        to_public_keys(nodes.to_owned(), fbas)
    } else {
        Vec::default()
    };
    let mut rankings: Vec<NodeRanking> = nodes
        .iter()
        .map(|&node| {
            (
                node,
                if with_pks {
                    pks[node].clone()
                } else {
                    PublicKey::default()
                },
                scores[node],
            )
        })
        .collect();
    rankings.sort_by(|x, y| scores[y.0].partial_cmp(&scores[x.0]).unwrap());
    rankings
}

/// Gets a list of (id, score, reward) and returns a list of (id, pk, score, reward) sorted by
/// score
pub fn create_reward_report(
    id_score_reward: Vec<(NodeId, Score, Reward)>,
    fbas: &Fbas,
    with_pks: bool,
) -> Vec<NodeReward> {
    let nodes: Vec<NodeId> = id_score_reward.iter().map(|n| n.0).collect();
    let scores: Vec<Score> = id_score_reward.iter().map(|s| s.1).collect();
    let rewards: Vec<Score> = id_score_reward.iter().map(|r| r.2).collect();

    let pks = if with_pks {
        to_public_keys(nodes.to_owned(), fbas)
    } else {
        Vec::default()
    };
    let mut rewards: Vec<NodeReward> = nodes
        .iter()
        .map(|&node| {
            (
                node,
                if with_pks {
                    pks[node].clone()
                } else {
                    PublicKey::default()
                },
                scores[node],
                rewards[node],
            )
        })
        .collect();
    rewards.sort_by(|x, y| scores[y.0].partial_cmp(&scores[x.0]).unwrap());
    rewards
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn read_fbas_from_str() -> Fbas {
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
                        "node4"
                    ]
                }
            }]"#;
        Fbas::from_json_str(&input)
    }

    #[test]
    fn node_rankings_output_is_correct() {
        let fbas = read_fbas_from_str();
        let nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let scores = compute_node_rank_for_fbas(&nodes, &fbas);
        let actual = create_node_ranking_report(&nodes, scores.to_owned(), &fbas, true);
        let expected = vec![
            (0, String::from("node0"), scores[0]),
            (1, String::from("node1"), scores[1]),
            (2, String::from("node2"), scores[2]),
            (4, String::from("node4"), scores[4]),
            (3, String::from("node3"), scores[3]),
        ];
        assert_eq!(expected, actual);
    }
    #[test]
    fn node_rewards_output_is_correct() {
        let fbas = read_fbas_from_str();
        let nodes: Vec<NodeId> = (0..fbas.all_nodes().len()).collect();
        let reward = 10.0;
        let dist = graph_theory_distribution(&nodes, &fbas, reward);
        let actual = create_reward_report(dist.to_owned(), &fbas, true);
        let expected = vec![
            (0, String::from("node0"), dist[0].1, dist[0].2),
            (1, String::from("node1"), dist[1].1, dist[1].2),
            (2, String::from("node2"), dist[2].1, dist[2].2),
            (4, String::from("node4"), dist[4].1, dist[4].2),
            (3, String::from("node3"), dist[3].1, dist[3].2),
        ];
        assert_eq!(expected, actual);
    }
}
