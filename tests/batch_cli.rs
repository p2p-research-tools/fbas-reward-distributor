use assert_cmd::Command;

#[test]
#[cfg_attr(not(feature = "measurements"), ignore)]
fn valid_batch_perf_experiments() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("performance_tests")?;
    cmd.arg("-r")
        .arg("1")
        .arg("-m")
        .arg("1")
        .arg("stellar")
        .arg("power-index-enum");
    // log writes to stderr by default
    cmd.assert().success();
    Ok(())
}

#[test]
#[cfg_attr(not(feature = "measurements"), ignore)]
fn no_fbas_type_in_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("approximation_tests")?;
    cmd.arg("-m")
        .arg("10")
        .arg("-j")
        .arg("4")
        .arg("-o")
        .arg("./");
    let output = cmd.output().expect("error executing command");
    assert!(!output.status.success());
    Ok(())
}
