use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn rank_only_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("node_influence")?;
    cmd.arg("rank")
        .arg("test_data/trivial.json")
        .arg("exact-power-index");
    cmd.assert().success().stdout(predicate::str::contains(
        "List of Rankings as (NodeId, PK, Score):",
    ));
    Ok(())
}

#[test]
fn dist_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("node_influence")?;
    cmd.arg("distribute")
        .arg("test_data/trivial.json")
        .arg("node-rank");
    cmd.assert().success().stdout(predicate::str::contains(
        "List of Distributions as (NodeId, PK, Score, Reward):",
    ));
    Ok(())
}

#[test]
fn invalid_command_without_alg() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("node_influence")?;
    cmd.arg("distribute")
        .arg("-r")
        .arg("50")
        .arg("test_data/trivial.json");
    let output = cmd.output().expect("error executing command");
    assert!(!output.status.success());
    Ok(())
}

#[test]
fn approx_command_without_samples() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("node_influence")?;
    cmd.arg("distribute")
        .arg("test_data/trivial.json")
        .arg("approx-power-index");
    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
    Ok(())
}
