use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn rank_only_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("node_influence")?;
    cmd.arg("rank")
        .arg("-a")
        .arg("exact-powerindex")
        .arg("test_data/correct_trivial.json");
    cmd.assert().success().stdout(predicate::str::contains(
        "List of Rankings as (NodeId, PK, Score):",
    ));
    Ok(())
}

#[test]
fn dist_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("node_influence")?;
    cmd.arg("distribute")
        .arg("-a")
        .arg("noderank")
        .arg("test_data/correct_trivial.json");
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
        .arg("test_data/correct_trivial.json");
    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
    Ok(())
}

#[test]
fn approx_command_without_samples() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("node_influence")?;
    cmd.arg("distribute")
        .arg("-a")
        .arg("approx-powerindex")
        .arg("test_data/correct_trivial.json");
    cmd.assert().failure().stderr(predicate::str::contains(
        "-a approx-powerindex requires the number of samples",
    ));
    Ok(())
}
