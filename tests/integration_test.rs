use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn setup_test_env() -> TempDir {
    let dir = TempDir::new().unwrap();
    unsafe {
        std::env::set_var("POMODORO_CLI_TEST_DIR", dir.path());
    }
    std::thread::sleep(Duration::from_millis(10));

    let timer_file = dir.path().join("pomodoro-cli-info.json");
    let _ = std::fs::remove_file(&timer_file);
    dir
}

fn create_cmd(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("pomodoro-cli").unwrap();
    cmd.env("POMODORO_CLI_TEST_DIR", temp_dir.path());
    cmd
}

#[test]
#[serial]
fn test_start_and_status() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("25m")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(100));

    create_cmd(&temp).arg("status").assert().success().stdout(
        predicate::str::contains("24:")
            .or(predicate::str::contains("25:"))
            .or(predicate::str::contains("Running")),
    );

    create_cmd(&temp).arg("stop").assert().success();
}

#[test]
#[serial]
fn test_start_pause_resume() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("10m")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(100));

    create_cmd(&temp).arg("pause").assert().success();

    create_cmd(&temp)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Paused"));

    create_cmd(&temp).arg("pause").assert().success();

    create_cmd(&temp)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(":"));

    create_cmd(&temp).arg("stop").assert().success();
}

#[test]
#[serial]
fn test_json_output() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("5m")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(100));

    let output = create_cmd(&temp)
        .arg("status")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(json["class"], "running");
    assert!(json["percentage"].as_f64().unwrap() >= 0.0);

    create_cmd(&temp).arg("stop").assert().success();
}

#[test]
#[serial]
fn test_invalid_duration() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));
}

#[test]
#[serial]
fn test_add_time() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("10m")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(100));

    create_cmd(&temp)
        .arg("start")
        .arg("--add")
        .arg("5m")
        .assert()
        .success();

    create_cmd(&temp).arg("status").assert().success().stdout(
        predicate::str::contains("14:")
            .or(predicate::str::contains("15:"))
            .or(predicate::str::contains("13:")),
    );

    create_cmd(&temp).arg("stop").assert().success();
}

#[test]
#[serial]
fn test_start_with_message() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("5m")
        .arg("--message")
        .arg("Test task")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(100));

    create_cmd(&temp)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Test task"));

    create_cmd(&temp).arg("stop").assert().success();
}

#[test]
#[serial]
fn test_stop_without_timer() {
    let temp = setup_test_env();

    create_cmd(&temp).arg("stop").assert().success();
}

#[test]
#[serial]
fn test_pause_without_timer() {
    let temp = setup_test_env();

    create_cmd(&temp).arg("pause").assert().success();
}

#[test]
#[serial]
fn test_status_without_timer() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Time is up!"));
}

#[test]
#[serial]
fn test_different_duration_formats() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("1h30m")
        .assert()
        .success();

    create_cmd(&temp).arg("stop").assert().success();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("45")
        .assert()
        .success();

    create_cmd(&temp).arg("stop").assert().success();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("1:30")
        .assert()
        .success();

    create_cmd(&temp).arg("stop").assert().success();
}

#[test]
#[serial]
#[ignore]
fn test_short_timer_completes() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("2s")
        .arg("--wait")
        .arg("--silent")
        .timeout(Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
#[serial]
fn test_segmented_time_format() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("1h30m")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(100));

    create_cmd(&temp)
        .arg("status")
        .arg("--time-format")
        .arg("segmented")
        .assert()
        .success()
        .stdout(predicate::str::contains("1h").or(predicate::str::contains("89m")));

    create_cmd(&temp).arg("stop").assert().success();
}

#[test]
#[serial]
fn test_digital_time_format() {
    let temp = setup_test_env();

    create_cmd(&temp)
        .arg("start")
        .arg("--duration")
        .arg("25m")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(100));

    create_cmd(&temp)
        .arg("status")
        .arg("--time-format")
        .arg("digital")
        .assert()
        .success()
        .stdout(predicate::str::contains(":"));

    create_cmd(&temp).arg("stop").assert().success();
}
