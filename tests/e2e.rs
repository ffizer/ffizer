extern crate assert_cmd;
extern crate failure;
extern crate ffizer;
extern crate tempfile;
extern crate test_generator;

use assert_cmd::prelude::*;
use failure::Error;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::tempdir;

mod dir_diff;

test_generator::test_expand_paths! { test_local_sample; "tests/test_*" }

fn test_local_sample(dir_name: &str) {
    assert!(test_local_sample_impl(dir_name).is_ok());
}

fn test_local_sample_impl(dir_name: &str) -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let sample_path = PathBuf::from(dir_name);
    let template_path = sample_path.join("template");
    let expected_path = sample_path.join("expected");
    let actual_path = tmp_dir.path().join("my-project").to_path_buf();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
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
fn empty_template() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let template_path = tmp_dir.path().join("t0_template");
    let expected_path = tmp_dir.path().join("t0_expected");
    let actual_path = tmp_dir.path().join("t0_actual");

    fs::create_dir_all(&template_path)?;
    fs::create_dir_all(&expected_path)?;

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
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

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
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

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_master() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
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

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_commitsha1() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("a476767b3ea4cde604d28761c4a2f8e4a31198e0")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_tag() -> Result<(), Error> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--x-always_default_value")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("1.1.0")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .assert()
        .success();

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}
