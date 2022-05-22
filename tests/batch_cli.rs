use assert_cmd::Command;
use predicates::prelude::*;

#[test]
#[ignore]
// ignore for now because binary only available when feature is active
// causes coverage test to fail
fn valid_batch_perf_experiments_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("batch_performance_eval")?;
    cmd.arg("-r").arg("1").arg("-m").arg("1").arg("stellar");
    cmd.assert().success().stdout(predicate::str::contains(
        "Starting performance measurements for Stellar like FBAS with upto 1 nodes.\n Performing 1 iterations per FBAS.",
    ));
    Ok(())
}

#[test]
#[ignore]
fn no_fbas_type_in_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("batch_error_eval")?;
    cmd.arg("--features")
        .arg("measurements")
        .arg("-m")
        .arg("10")
        .arg("-j")
        .arg("4")
        .arg("-o")
        .arg("./");
    let output = cmd.output().expect("error executing command");
    assert!(!output.status.success());
    Ok(())
}
