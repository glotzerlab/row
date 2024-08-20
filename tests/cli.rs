// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use serial_test::parallel;
use std::fs;

use row::DATA_DIRECTORY_NAME;

/// Create a sample workflow and workspace to use with the tests.
fn setup_sample_workflow(
    temp: &TempDir,
    n: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();

    for i in 0..n {
        let name = format!("dir{i}");
        let directory = temp.child("workspace").child(&name);
        directory.create_dir_all().unwrap();
        directory
            .child("v.json")
            .write_str(&format!("{{\"v\": {i}, \"v2\": {}}}", i / 2))
            .unwrap();

        result.push(name);
    }

    temp.child("workflow.toml").write_str(
        r#"
[workspace]
value_file = "v.json"

[[action]]
name = "one"
command = "touch workspace/{directory}/one"
products = ["one"]

[[action]]
name = "two"
command = "touch workspace/{directory}/two"
products = ["two"]
previous_actions = ["one"]
"#,
    )?;

    Ok(result)
}

/// Complete the first n directories for action in the sample workspace.
fn complete_action(
    action: &str,
    temp: &TempDir,
    n: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..n {
        let name = format!("dir{i}");
        temp.child("workspace").child(&name).child(action).touch()?;
    }

    Ok(())
}

#[test]
#[parallel]
fn requires_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("requires a subcommand"));

    Ok(())
}

#[test]
#[parallel]
fn no_workflow_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    let temp = TempDir::new()?;

    cmd.args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("workflow.toml not found"));

    Ok(())
}

#[test]
#[parallel]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;

    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: row"));

    Ok(())
}

#[test]
#[parallel]
fn empty_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    cmd.env("ROW_HOME", "/not/a/path");

    let temp = TempDir::new()?;
    temp.child("workflow.toml").touch()?;
    temp.child("workspace").create_dir_all()?;

    cmd.args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("No actions match"));

    Ok(())
}

#[test]
#[parallel]
fn status() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?);

    Ok(())
}

#[test]
#[parallel]
fn status_action_selection() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .arg("-a")
        .arg("one")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?.not());

    Ok(())
}

#[test]
#[parallel]
fn status_directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .arg("dir1")
        .arg("dir2")
        .arg("nodir")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +2 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +2")?)
        .stderr(predicate::str::contains("'nodir' not found in workspace"));

    Ok(())
}

#[test]
#[parallel]
fn status_directories_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .arg("-")
        .current_dir(temp.path())
        .write_stdin("dir1\ndir2\nnodir\n")
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +2 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +2")?)
        .stderr(predicate::str::contains("'nodir' not found in workspace"));

    Ok(())
}

#[test]
#[parallel]
fn scan() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?);

    complete_action("one", &temp, 8)?;
    complete_action("two", &temp, 4)?;

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?);

    let completed = temp.child(".row").child("completed");
    completed.assert(predicate::path::missing());

    Command::cargo_bin("row")?
        .arg("scan")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success();

    completed.assert(predicate::path::exists());
    assert_eq!(fs::read_dir(completed.path())?.count(), 1);

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +8 +0 +2 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +4 +0 +4 +2")?);

    assert_eq!(fs::read_dir(completed.path())?.count(), 0);

    Ok(())
}

#[test]
#[parallel]
fn scan_action() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?);

    complete_action("one", &temp, 8)?;
    complete_action("two", &temp, 4)?;

    Command::cargo_bin("row")?
        .arg("scan")
        .arg("-a")
        .arg("one")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success();

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +8 +0 +2 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +8 +2")?);

    Ok(())
}

#[test]
#[parallel]
fn scan_directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?);

    complete_action("one", &temp, 8)?;
    complete_action("two", &temp, 4)?;

    Command::cargo_bin("row")?
        .arg("scan")
        .arg("dir5")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success();

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +1 +0 +9 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +1 +9")?);

    Ok(())
}

#[test]
#[parallel]
fn submit() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .arg("submit")
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success();

    Command::cargo_bin("row")?
        .args(["show", "status"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +10 +0 +0 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +10 +0")?);

    Ok(())
}

#[test]
#[parallel]
fn directories_no_action() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 4);

    Command::cargo_bin("row")?
        .args(["show", "directories"])
        .args(["--cluster", "none"])
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicates::str::diff("dir0\ndir1\ndir2\ndir3\n"));

    Ok(())
}

#[test]
#[parallel]
fn directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "directories"])
        .args(["--cluster", "none"])
        .args(["--action", "one"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^Directory Status +Job ID")?)
        .stdout(predicate::str::is_match("(?m)^dir0 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir1 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir2 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir3 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir4 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir5 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir6 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir7 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir8 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir9 *eligible *$")?);

    Ok(())
}

#[test]
#[parallel]
fn directories_select_directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "directories"])
        .args(["--cluster", "none"])
        .args(["--action", "one"])
        .arg("dir3")
        .arg("dir9")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^Directory Status +Job ID")?)
        .stdout(predicate::str::is_match("(?m)^dir0 *eligible *$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir1 *eligible *$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir2 *eligible *$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir3 *eligible *$")?)
        .stdout(predicate::str::is_match("(?m)^dir4 *eligible *$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir5 *eligible *$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir6 *eligible *$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir7 *eligible *$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir8 *eligible *$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir9 *eligible *$")?);

    Ok(())
}

#[test]
#[parallel]
fn directories_no_header() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "directories"])
        .args(["--cluster", "none"])
        .args(["--action", "one"])
        .arg("--no-header")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^Directory Status")?.not());

    Ok(())
}

#[test]
#[parallel]
fn directories_value() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "directories"])
        .args(["--cluster", "none"])
        .args(["--value", "/v"])
        .args(["--value", "/v2"])
        .args(["--action", "one"])
        .arg("dir3")
        .arg("dir9")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::is_match(
            "(?m)^Directory +Status +Job ID +/v +/v2",
        )?)
        .stdout(predicate::str::is_match("(?m)^dir3 +eligible +3 +1$")?)
        .stdout(predicate::str::is_match("(?m)^dir9 +eligible +9 +4$")?);

    Ok(())
}

#[test]
#[parallel]
fn directories_short() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 4);

    Command::cargo_bin("row")?
        .args(["show", "directories"])
        .args(["--cluster", "none"])
        .args(["--action", "one"])
        .arg("--short")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicates::str::diff("dir0\ndir1\ndir2\ndir3\n"));

    Ok(())
}

#[test]
#[parallel]
fn directories_short_no_action() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(["show", "directories"])
        .args(["--cluster", "none"])
        .arg("--short")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--action"));

    Ok(())
}

#[test]
#[parallel]
fn show_cluster() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    Command::cargo_bin("row")?
        .args(["show", "cluster"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"name = "none""#));

    Ok(())
}
#[test]
#[parallel]
fn show_cluster_short() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    Command::cargo_bin("row")?
        .args(["show", "cluster"])
        .args(["--cluster", "none"])
        .arg("--short")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::eq("none\n"));

    Ok(())
}

#[test]
#[parallel]
fn show_launchers() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    Command::cargo_bin("row")?
        .args(["show", "launchers"])
        .args(["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"executable = "mpirun""#));

    Ok(())
}
#[test]
#[parallel]
fn show_launchers_short() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    Command::cargo_bin("row")?
        .args(["show", "launchers"])
        .args(["--cluster", "none"])
        .arg("--short")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .env("ROW_HOME", "/not/a/path")
        .assert()
        .success()
        .stdout(predicate::str::contains("mpi"))
        .stdout(predicate::str::contains("openmp\n"));

    Ok(())
}

#[test]
#[parallel]
fn init_conflicting_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    let temp = TempDir::new()?;

    cmd.args(["init"])
        .arg("--signac")
        .args(["--workspace", "test"])
        .current_dir(temp.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));

    Ok(())
}

#[test]
#[parallel]
fn init_invalid_path() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    let temp = TempDir::new()?;

    cmd.args(["init"])
        .args(["--workspace", "/test/one"])
        .arg(".")
        .current_dir(temp.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("must be a relative"));

    Ok(())
}

#[test]
#[parallel]
fn init_workflow_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    let temp = TempDir::new()?;
    temp.child("workflow.toml").touch()?;

    cmd.args(["init"]).arg(".").current_dir(temp.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("project already exists"));

    Ok(())
}

#[test]
#[parallel]
fn init_parent_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    let temp = TempDir::new()?;
    temp.child("workflow.toml").touch()?;

    let subdir = temp.child("subdir");
    subdir.create_dir_all()?;

    cmd.args(["init"]).arg(".").current_dir(subdir.path());
    cmd.assert().failure().stderr(predicate::str::contains(
        "project already exists in the parent",
    ));

    Ok(())
}

#[test]
#[parallel]
fn init_cache_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    let temp = TempDir::new()?;
    temp.child(DATA_DIRECTORY_NAME).touch()?;

    cmd.args(["init"]).arg(".").current_dir(temp.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cache directory"))
        .stderr(predicate::str::contains("already exists"));

    Ok(())
}

#[test]
#[parallel]
fn init() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    let temp = TempDir::new()?;

    cmd.args(["init"]).arg(".").current_dir(temp.path());
    cmd.assert().success();

    temp.child("workspace").assert(predicate::path::is_dir());
    temp.child("workflow.toml")
        .assert(predicate::path::is_file());

    Ok(())
}
