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
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
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
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[cfg(any(test_remote))]
#[test]
fn test_2() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let template_path = PathBuf::from("./tests/test_2/template");
    let expected_path = PathBuf::from("./tests/test_2/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&template_path)?;
    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[test]
fn test_3() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let template_path = PathBuf::from("./tests/test_3/template");
    let expected_path = PathBuf::from("./tests/test_3/expected");
    let actual_path = tmp_dir.path().join("test_3").to_path_buf();

    fs::create_dir_all(&template_path)?;
    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[test]
fn test_1_subfolder() -> Result<(), Error> {
    let source_subfolder = "dir_1";
    let tmp_dir = tempdir()?;
    let template_path = PathBuf::from("./tests/test_1/template");
    let expected_path = PathBuf::from("./tests/test_1/expected").join(source_subfolder);
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&template_path)?;
    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .arg("--source-subfolder")
        .arg(source_subfolder)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[cfg(any(test_remote))]
#[test]
fn test_1_remote_master() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[cfg(any(test_remote))]
#[test]
fn test_1_remote_commitsha1() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("8cab693bbf2eb4f8291ede174d8625d8d21e7b92")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[cfg(any(test_remote))]
#[test]
fn test_1_remote_tag() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    fs::create_dir_all(&expected_path)?;

    Command::main_binary()?
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("1.0.0")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}
