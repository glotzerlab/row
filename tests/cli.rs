use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;

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
fn requires_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("requires a subcommand"));

    Ok(())
}

#[test]
fn no_workflow_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;
    let temp = TempDir::new()?;

    cmd.args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("workflow.toml not found"));

    Ok(())
}

#[test]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;

    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: row"));

    Ok(())
}

#[test]
fn empty_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("row")?;

    let temp = TempDir::new()?;
    temp.child("workflow.toml").touch()?;
    temp.child("workspace").create_dir_all()?;

    cmd.args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path());
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("No actions match"));

    Ok(())
}

#[test]
fn status() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?);

    Ok(())
}

#[test]
fn status_action_selection() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .arg("-a")
        .arg("one")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?.not());

    Ok(())
}

#[test]
fn status_directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .arg("dir1")
        .arg("dir2")
        .arg("nodir")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +2 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +2")?)
        .stderr(predicate::str::contains("'nodir' not found in workspace"));

    Ok(())
}

#[test]
fn status_directories_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .arg("-")
        .current_dir(temp.path())
        .write_stdin("dir1\ndir2\nnodir\n")
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +2 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +2")?)
        .stderr(predicate::str::contains("'nodir' not found in workspace"));

    Ok(())
}

#[test]
fn scan() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +0 +0 +10 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +0 +10")?);

    complete_action("one", &temp, 8)?;
    complete_action("two", &temp, 4)?;

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
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
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +8 +0 +2 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +4 +0 +4 +2")?);

    assert_eq!(fs::read_dir(completed.path())?.count(), 0);

    Ok(())
}

#[test]
fn scan_action() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
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
        .assert()
        .success();

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +8 +0 +2 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +8 +2")?);

    Ok(())
}

#[test]
fn scan_directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
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
        .assert()
        .success();

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +1 +0 +9 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +1 +9")?);

    Ok(())
}

#[test]
fn submit() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .arg("submit")
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success();

    Command::cargo_bin("row")?
        .args(&["show", "status"])
        .args(&["--cluster", "none"])
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^one +10 +0 +0 +0")?)
        .stdout(predicate::str::is_match("(?m)^two +0 +0 +10 +0")?);

    Ok(())
}

#[test]
fn directories_no_action() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "directories"])
        .args(&["--cluster", "none"])
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));

    Ok(())
}

#[test]
fn directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "directories"])
        .args(&["--cluster", "none"])
        .arg("one")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^Directory Status")?)
        .stdout(predicate::str::is_match("(?m)^dir0 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir1 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir2 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir3 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir4 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir5 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir6 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir7 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir8 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir9 *eligible$")?);

    Ok(())
}

#[test]
fn directories_select_directories() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "directories"])
        .args(&["--cluster", "none"])
        .arg("one")
        .arg("dir3")
        .arg("dir9")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^Directory Status")?)
        .stdout(predicate::str::is_match("(?m)^dir0 *eligible$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir1 *eligible$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir2 *eligible$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir3 *eligible$")?)
        .stdout(predicate::str::is_match("(?m)^dir4 *eligible$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir5 *eligible$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir6 *eligible$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir7 *eligible$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir8 *eligible$")?.not())
        .stdout(predicate::str::is_match("(?m)^dir9 *eligible$")?);

    Ok(())
}

#[test]
fn directories_no_header() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "directories"])
        .args(&["--cluster", "none"])
        .arg("one")
        .arg("--no-header")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^Directory Status")?.not());

    Ok(())
}

#[test]
fn directories_value() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let _ = setup_sample_workflow(&temp, 10);

    Command::cargo_bin("row")?
        .args(&["show", "directories"])
        .args(&["--cluster", "none"])
        .args(&["--value", "/v"])
        .args(&["--value", "/v2"])
        .arg("one")
        .arg("dir3")
        .arg("dir9")
        .current_dir(temp.path())
        .env_remove("ROW_COLOR")
        .env_remove("CLICOLOR")
        .assert()
        .success()
        .stdout(predicate::str::is_match("(?m)^Directory +Status +/v +/v2")?)
        .stdout(predicate::str::is_match("(?m)^dir3 +eligible +3 +1$")?)
        .stdout(predicate::str::is_match("(?m)^dir9 +eligible +9 +4$")?);

    Ok(())
}
