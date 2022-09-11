use assert_cmd::Command;
use ffizer::tools::dir_diff_list;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use test_generator::test_resources;

#[test_resources("tests/data/template_*")]
#[allow(clippy::assertions_on_constants)]
fn run_test_samples(template_path: &str) {
    let t = do_run_test_samples(template_path);
    if let Err(e) = t {
        dbg!(e);
        assert!(false);
    }
}

pub fn do_run_test_samples(template_path: &str) -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("test-samples")
        .arg("--source")
        .arg(template_path)
        .ok()?;
    Ok(())
}

/// Are the contents of two directories same?
pub fn assert_is_same<A: AsRef<Path>, B: AsRef<Path>>(
    actual_base: A,
    expected_base: B,
    output: &std::process::Output,
) -> Result<(), Box<dyn Error>> {
    let diffs = dir_diff_list::search_diff(actual_base, expected_base)?;
    if !diffs.is_empty() || !output.status.success() {
        dbg!(output);
    }
    assert_eq!(diffs, vec![]);
    assert_eq!(output.status.success(), true);
    Ok(())
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
    let template_path = PathBuf::from("./tests/data/template_1");
    let expected_path =
        PathBuf::from("./tests/data/template_1/.ffizer.samples.d/my-project.expected")
            .join(source_subfolder);
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
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("test-samples")
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .ok()?;
    Ok(())
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_commitsha1() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("test-samples")
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("3ab3bc67b5fab58ceecc031f7ed0eb29c0e0fff8")
        .ok()?;
    Ok(())
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_tag() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("test-samples")
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("1.2.0")
        .ok()?;
    Ok(())
}

// reproduce https://github.com/ffizer/ffizer/issues/195
#[test]
fn log_should_report_error() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let sample_path = PathBuf::from("tests/data/log_error");
    let template_path = sample_path.join("template");
    let actual_path = tmp_dir.path().join("my-project");

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
            predicate::str::contains("source: RenderError")
                .and(predicate::str::contains("reason: InvalidSyntax,")),
        )
        .failure();
    Ok(())
}
