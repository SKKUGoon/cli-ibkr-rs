use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn ibkrctl() -> Command {
    let binary = env!("CARGO_BIN_EXE_ibkrctl");
    let mut command = Command::new(binary);
    command.env_clear();
    command
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ibkrctl-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

#[test]
fn env_command_shows_unset_values_without_required_config() {
    let dir = temp_dir("unset");
    let output = ibkrctl()
        .current_dir(&dir)
        .arg("env")
        .output()
        .expect("ibkrctl should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("IBKR_CONSUMER_KEY=<unset>"));
    assert!(stdout.contains("IBKR_ACCESS_TOKEN=<unset>"));
    assert!(stdout.contains("IBKR_BASE_URL=<unset>"));
}

#[test]
fn env_file_loads_explicit_file_and_truncates_secrets() {
    let dir = temp_dir("explicit");
    let env_file = dir.join("custom.env");
    fs::write(
        &env_file,
        [
            "IBKR_CONSUMER_KEY=file-consumer-secret",
            "IBKR_ACCESS_TOKEN=file-access-token",
            "IBKR_BASE_URL=https://example.test/v1/api",
        ]
        .join("\n"),
    )
    .expect("env file should be written");

    let output = ibkrctl()
        .arg("--env-file")
        .arg(&env_file)
        .arg("env")
        .output()
        .expect("ibkrctl should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("IBKR_CONSUMER_KEY=file...cret"));
    assert!(stdout.contains("IBKR_ACCESS_TOKEN=file...oken"));
    assert!(stdout.contains("IBKR_BASE_URL=https://example.test/v1/api"));
    assert!(!stdout.contains("file-consumer-secret"));
    assert!(!stdout.contains("file-access-token"));
}

#[test]
fn process_environment_wins_over_explicit_env_file() {
    let dir = temp_dir("precedence");
    let env_file = dir.join("custom.env");
    fs::write(&env_file, "IBKR_CONSUMER_KEY=file-consumer-secret\n")
        .expect("env file should be written");

    let output = ibkrctl()
        .env("IBKR_CONSUMER_KEY", "process-consumer-secret")
        .arg("--env-file")
        .arg(&env_file)
        .arg("env")
        .output()
        .expect("ibkrctl should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("IBKR_CONSUMER_KEY=proc...cret"));
    assert!(!stdout.contains("file-consumer-secret"));
}

#[test]
fn missing_explicit_env_file_fails_fast() {
    let dir = temp_dir("missing");
    let missing = dir.join("missing.env");

    let output = ibkrctl()
        .arg("--env-file")
        .arg(&missing)
        .arg("env")
        .output()
        .expect("ibkrctl should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("failed to load env file"));
    assert!(stderr.contains("missing.env"));
}

#[test]
fn default_dotenv_search_still_loads_from_current_directory() {
    let dir = temp_dir("default");
    fs::write(dir.join(".env"), "IBKR_REALM=paper\n").expect(".env should be written");

    let output = ibkrctl()
        .current_dir(&dir)
        .arg("env")
        .output()
        .expect("ibkrctl should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("IBKR_REALM=paper"));
}

#[test]
fn version_flag_prints_package_version() {
    let output = ibkrctl()
        .arg("--version")
        .output()
        .expect("ibkrctl should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(stdout.trim(), "ibkrctl 0.3.0\nworker 0.3.0\ncore 0.3.0");
}
