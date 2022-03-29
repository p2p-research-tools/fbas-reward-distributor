use crate::{ErrorDataPoint, InputDataPoint, PerfDataPoint};
use fbas_analyzer::*;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub enum FbasType {
    /// Make FBAS that looks like Stellar's top tier: every 3 top-tier nodes are organised as an
    /// inner_quorum set of the top-tier quorum set.
    Stellar,
    /// Full symmetric top tier
    MobileCoin,
    /// Non symmetric top tier
    NonSymmetric,
}

impl FbasType {
    pub fn node_increments(&self) -> usize {
        match self {
            FbasType::MobileCoin => 1,
            FbasType::Stellar => 3,
            FbasType::NonSymmetric => 1,
        }
    }
    pub fn make_one(&self, top_tier_size: usize) -> Fbas {
        match self {
            FbasType::MobileCoin => make_almost_ideal_fbas(top_tier_size),
            FbasType::Stellar => make_almost_ideal_stellarlike_fbas(top_tier_size),
            FbasType::NonSymmetric => make_almost_ideal_fbas(top_tier_size),
        }
    }
}

#[derive(Debug)]
pub enum Task {
    ReusePerfData(PerfDataPoint),
    ReuseErrorData(ErrorDataPoint),
    Analyze(InputDataPoint),
}
impl Task {
    pub fn label(&self) -> usize {
        match self {
            Task::ReusePerfData(output) => output.top_tier_size,
            Task::Analyze(input) => input.top_tier_size,
            Task::ReuseErrorData(output) => output.top_tier_size,
        }
    }
}

fn make_almost_ideal_fbas(top_tier_size: usize) -> Fbas {
    let quorum_set = QuorumSet {
        validators: (0..top_tier_size).collect(),
        threshold: simulation::qsc::calculate_67p_threshold(top_tier_size),
        inner_quorum_sets: vec![],
    };
    let mut fbas = Fbas::new();
    for _ in 0..top_tier_size {
        fbas.add_generic_node(quorum_set.clone());
    }
    fbas
}

fn make_almost_ideal_stellarlike_fbas(top_tier_size: usize) -> Fbas {
    assert!(
        top_tier_size % 3 == 0,
        "Nodes in the Stellar network top tier always come in groups of (at least) 3..."
    );
    let mut quorum_set = QuorumSet::new_empty();
    for org_id in 0..top_tier_size / 3 {
        let validators = vec![org_id * 3, org_id * 3 + 1, org_id * 3 + 2];
        quorum_set.inner_quorum_sets.push(QuorumSet {
            validators,
            threshold: 2,
            inner_quorum_sets: vec![],
        });
    }
    quorum_set.threshold = simulation::qsc::calculate_67p_threshold(top_tier_size / 3);
    let mut fbas = Fbas::new();
    for _ in 0..top_tier_size {
        fbas.add_generic_node(quorum_set.clone());
    }
    fbas
}
