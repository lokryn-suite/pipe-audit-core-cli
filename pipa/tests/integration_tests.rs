use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn test_health_command_runs() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("health").assert().success();
}

#[test]
fn test_contract_list_command_runs() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("contract")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_profile_list_command_runs() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("profile")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_run_without_args_fails() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("run")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Must specify either contract name or --all"));
}

#[test]
fn test_run_with_both_contract_and_all_fails() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("run")
        .arg("my_contract")
        .arg("--all")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Cannot specify both contract name and --all"));
}

#[test]
fn test_no_command_shows_message() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("No command specified"));
}

#[test]
fn test_verbose_flag_accepted() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("-v")
        .arg("health")
        .assert()
        .success();
}

#[test]
fn test_contract_validate_requires_file() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("contract")
        .arg("validate")
        .assert()
        .failure();
}

#[test]
fn test_contract_show_requires_name() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("contract")
        .arg("show")
        .assert()
        .failure();
}

#[test]
fn test_profile_test_requires_name() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("profile")
        .arg("test")
        .assert()
        .failure();
}

#[test]
fn test_logs_verify_accepts_date() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("logs")
        .arg("verify")
        .arg("--date")
        .arg("2025-01-15")
        .assert()
        .success();
}

#[test]
fn test_logs_verify_accepts_all() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("logs")
        .arg("verify")
        .arg("--all")
        .assert()
        .success();
}

#[test]
fn test_init_command_creates_structure() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success();

    // Verify that init created expected directories
    assert!(temp_dir.path().join("contracts").exists());
}

#[test]
fn test_run_single_contract_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("pipa").unwrap();

    // Create contracts directory but no contract file
    fs::create_dir_all(temp_dir.path().join("contracts")).unwrap();

    cmd.current_dir(&temp_dir)
        .arg("run")
        .arg("nonexistent")
        .assert()
        .success()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_contract_validate_with_file() {
    let temp_dir = TempDir::new().unwrap();
    let contracts_dir = temp_dir.path().join("contracts");
    fs::create_dir_all(&contracts_dir).unwrap();

    // Create a minimal valid test contract file
    let contract_file = contracts_dir.join("test.toml");
    let contract_content = r#"[contract]
name = "test"
version = "1.0"
tags = []

[[columns]]
name = "id"
validation = []
"#;
    fs::write(&contract_file, contract_content).unwrap();

    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("contract")
        .arg("validate")
        .arg("contracts/test.toml")
        .assert()
        .success();
}

#[test]
fn test_contract_show_existing() {
    let temp_dir = TempDir::new().unwrap();
    let contracts_dir = temp_dir.path().join("contracts");
    fs::create_dir_all(&contracts_dir).unwrap();

    // Create a minimal valid test contract file
    let contract_content = r#"[contract]
name = "test"
version = "1.0"
tags = []

[[columns]]
name = "id"
validation = []
"#;
    fs::write(contracts_dir.join("test.toml"), contract_content).unwrap();

    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("contract")
        .arg("show")
        .arg("test")
        .assert()
        .success();
}

#[test]
fn test_verbose_flag_with_different_commands() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("--verbose")
        .arg("contract")
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_short_verbose_flag_with_different_commands() {
    let mut cmd = Command::cargo_bin("pipa").unwrap();
    cmd.arg("-v")
        .arg("contract")
        .arg("list")
        .assert()
        .success();
}
