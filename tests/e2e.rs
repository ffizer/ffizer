extern crate assert_cmd;
extern crate failure;
extern crate ffizer;
extern crate tempfile;

use assert_cmd::prelude::*;
use failure::Error;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::tempdir;

mod dir_diff;

#[test]
fn empty_template() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let template_path = tmp_dir.path().join("t0_template");
    let expected_path = tmp_dir.path().join("t0_expected");
    let actual_path = tmp_dir.path().join("t0_actual");

    fs::create_dir_all(&template_path)?;
    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[test]
fn test_1() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let template_path = PathBuf::from("./tests/test_1/template");
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&template_path)?;
    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .stdout(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[test]
fn test_1_remote_master() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/davidB/ffizer_demo_template.git")
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[test]
fn test_1_remote_commitsha1() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/davidB/ffizer_demo_template.git")
        .arg("--rev")
        .arg("8cab693bbf2eb4f8291ede174d8625d8d21e7b92")
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[test]
fn test_1_remote_tag() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/davidB/ffizer_demo_template.git")
        .arg("--rev")
        .arg("1.0.0")
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}
