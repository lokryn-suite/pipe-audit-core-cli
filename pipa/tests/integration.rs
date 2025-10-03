use assert_cmd::Command;

#[test]
fn runs_example_contract() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("contract")
       .arg("validate")
       .arg("examples/contracts/example.toml")
       .assert()
       .success();
}

