use assert_cmd::Command;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use test_generator::test_resources;
mod dir_diff_list;

/// Are the contents of two directories same?
pub fn assert_is_same<A: AsRef<Path>, B: AsRef<Path>>(
    actual_base: A,
    expected_base: B,
    output: &std::process::Output,
) -> Result<(), Box<dyn Error>> {
    let diffs = dir_diff_list::search_diff(actual_base, expected_base)?;
    if !diffs.is_empty() {
        dbg!(output);
    }
    assert_eq!(diffs, vec![]);
    Ok(())
}

#[test_resources("tests/data/test_*")]
fn test_local_sample_keep(dir_name: &str) {
    assert!(test_local_sample_impl(dir_name, "keep").is_ok());
}

#[test_resources("tests/data/test_*")]
fn test_local_sample_override(dir_name: &str) {
    assert!(test_local_sample_impl(dir_name, "override").is_ok());
}

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
    let mut process = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    process
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg(update_mode)
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap());
    process.assert().success();
    assert_is_same(&actual_path, &expected_path, &process.output()?)
}

#[test]
fn empty_template() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let template_path = tmp_dir.path().join("t0_template");
    let expected_path = tmp_dir.path().join("t0_expected");
    let actual_path = tmp_dir.path().join("t0_actual");

    fs::create_dir_all(&template_path)?;
    fs::create_dir_all(&expected_path)?;

    let mut process = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    process
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg("keep")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg(template_path.to_str().unwrap());
    process.assert().success();
    assert_is_same(&actual_path, &expected_path, &process.output()?)
}

#[test]
fn test_1_subfolder() -> Result<(), Box<dyn Error>> {
    let source_subfolder = "dir_1";
    let tmp_dir = tempdir()?;
    let template_path = PathBuf::from("./tests/data/test_1/template");
    let expected_path = PathBuf::from("./tests/data/test_1/expected").join(source_subfolder);
    let actual_path = tmp_dir.path().to_path_buf();

    let mut process = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    process
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
        .arg(source_subfolder);
    process.assert().success();
    assert_is_same(&actual_path, &expected_path, &process.output()?)
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_master() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/data/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    let mut process = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    process
        .arg("apply")
        .arg("--no-interaction")
        .arg("--confirm")
        .arg("never")
        .arg("--update-mode")
        .arg("keep")
        .arg("--destination")
        .arg(actual_path.to_str().unwrap())
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git");
    process.assert().success();
    assert_is_same(&actual_path, &expected_path, &process.output()?)
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_commitsha1() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/data/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    let mut process = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    process
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
        .arg("a476767b3ea4cde604d28761c4a2f8e4a31198e0");
    process.assert().success();
    assert_is_same(&actual_path, &expected_path, &process.output()?)
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_tag() -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let expected_path = PathBuf::from("./tests/data/test_1/expected");
    let actual_path = tmp_dir.path().to_path_buf();

    let mut process = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    process
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
        .arg("1.1.0");
    process.assert().success();
    assert_is_same(&actual_path, &expected_path, &process.output()?)
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
        .stderr(predicate::str::contains(
            "source: TemplateError(TemplateError { reason: InvalidSyntax",
        ))
        .failure();
    Ok(())
}

/// recursively copy a directory
/// based on https://stackoverflow.com/a/60406693/469066
pub fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        //println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            // println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        //println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        //println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}
