use assert_cmd::Command;
use ffizer::tools::copy;
use ffizer::tools::dir_diff_list;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use test_generator::test_resources;

/// Are the contents of two directories same?
pub fn assert_is_same<A: AsRef<Path>, B: AsRef<Path>>(
    actual_base: A,
    expected_base: B,
    output: &std::process::Output,
) -> Result<(), Box<dyn Error>> {
    let diffs = dir_diff_list::search_diff(actual_base, expected_base)?;
    dbg!(&output);
    if !diffs.is_empty() || !output.status.success() {
        dbg!(output);
    }
    assert_eq!(diffs, vec![]);
    assert_eq!(output.status.success(), true);
    Ok(())
}

#[test_resources("tests/data/test_*")]
fn test_local_sample_keep(dir_name: &str) {
    let t = test_local_sample_impl(dir_name, "keep");
    if let Err(e) = t {
        dbg!(e);
        assert!(false);
    }
}

// #[test_resources("tests/data/test_*")]
// fn test_local_sample_override(dir_name: &str) {
//     assert!(test_local_sample_impl(dir_name, "override").is_ok());
// }

fn test_local_sample_impl(dir_name: &str, update_mode: &str) -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let sample_path = PathBuf::from(dir_name);
    let template_path = sample_path.join("template");
    let expected_path = sample_path.join("expected");
    let existing_path = sample_path.join("existing");
    let actual_path = tmp_dir.path().join("my-project").to_path_buf();
    assert_eq!(false, actual_path.exists());

    if existing_path.exists() {
        copy(&existing_path, &actual_path)?;
    }
    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg(update_mode)
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .arg("-v")
        .arg("k2=v2_from_cli")
        .ok()?;
    assert_is_same(&actual_path, &expected_path, &output)
}

#[test]
fn empty_template() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let template_path = tmp_dir.path().join("t0_template");
    let expected_path = tmp_dir.path().join("t0_expected");
    let actual_path = tmp_dir.path().join("t0_actual");

    fs::create_dir_all(&template_path)?;
    fs::create_dir_all(&expected_path)?;

    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg("keep")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .ok()?;
    assert_is_same(&actual_path, &expected_path, &output)
}

#[test]
fn test_1_subfolder() -> Result<(), Box<dyn Error>> {
    let source_subfolder = "dir_1";
    let tmp_dir = tempdir()?;
    let template_path = PathBuf::from("./tests/data/test_1/template");
    let expected_path = PathBuf::from("./tests/data/test_1/expected").join(source_subfolder);
    let actual_path = tmp_dir.path().to_path_buf();

    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg("keep")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .arg("--source-subfolder")
        .arg(source_subfolder)
        .ok()?;
    assert_is_same(&actual_path, &expected_path, &output)
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_master() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/data/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg("keep")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .ok()?;
    assert_is_same(&actual_path, &expected_path, &output)
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_commitsha1() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/data/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg("keep")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("a476767b3ea4cde604d28761c4a2f8e4a31198e0")
        .ok()?;
    assert_is_same(&actual_path, &expected_path, &output)
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_tag() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/data/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg("keep")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("1.1.0")
        .ok()?;
    assert_is_same(&actual_path, &expected_path, &output)
}

// reproduce https://github.com/ffizer/ffizer/issues/195
#[test]
fn log_should_report_error() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let sample_path = PathBuf::from("tests/data/log_error");
    let template_path = sample_path.join("template");
    let actual_path = tmp_dir.path().join("my-project").to_path_buf();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .assert()
        .stderr(
            predicate::str::contains("source: TemplateError(")
                .and(predicate::str::contains("reason: InvalidSyntax,")),
        )
        .failure();
    Ok(())
}

#[test]
fn run_test_samples() -> Result<(), Box<dyn Error>> {
    let template_path = PathBuf::from("tests/data/template_2");

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("test-samples")
        .arg("--source")
        .arg(template_path.to_str().unwrap())
        .ok()?;
    Ok(())
}
