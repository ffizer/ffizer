use assert_cmd::Command;
use ffizer::tools::dir_diff_list;
use ffizer::PathPattern;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;
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
    let diffs = dir_diff_list::search_diff(
        actual_base,
        expected_base,
        &[PathPattern::from_str(".ffizer/version.txt")?],
    )?;
    if !diffs.is_empty() || !output.status.success() {
        dbg!(output);
    }
    assert_eq!(diffs, vec![]);
    assert_eq!(output.status.success(), true);
    Ok(())
}

mod test_reapply {
    use super::*;
    use rstest::*;
    use similar_asserts::assert_eq;

    #[rstest]
    fn single_template_case() -> Result<(), Box<dyn Error>> {
        let source_subfolder = "template_1";
        let tmp_dir = tempdir()?;
        let template_path = PathBuf::from("./tests/data");
        let expected_path =
            PathBuf::from("./tests/data/template_1/.ffizer.samples.d/my-project.expected");
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
        assert_is_same(&actual_path, &expected_path, &output)?;

        let tmp_dir_2 = tempdir()?;
        let reapply_path = tmp_dir_2.path().to_path_buf();

        Command::new("cp")
            .arg("-r")
            .arg(tmp_dir.path().join(".ffizer"))
            .arg(tmp_dir_2.path().join(".ffizer"))
            .ok()?;

        let output_reapply = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
            .arg("reapply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("keep")
            .arg("--destination")
            .arg(reapply_path.to_str().unwrap())
            .ok()?;
        assert_is_same(&reapply_path, &expected_path, &output_reapply)?;

        Ok(())
    }

    #[rstest]
    fn multi_template_case() -> Result<(), Box<dyn Error>> {
        let tmp_dir_apply = tempdir()?;
        let template_path = PathBuf::from("./tests/data/4compose");
        let source_subfolder_1 = "template_1";
        let source_subfolder_2 = "template_2";
        let expected_path = PathBuf::from("./tests/data/reapply/expected");
        let actual_path = tmp_dir_apply.path().to_path_buf();

        let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
            .arg("apply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("override")
            .arg("--destination")
            .arg(actual_path.to_str().unwrap())
            .arg("--source")
            .arg(template_path.to_str().unwrap())
            .arg("--source-subfolder")
            .arg(source_subfolder_2)
            .arg("-v")
            .arg("project_name=my-project")
            .ok()?;

        assert_eq!(output.status.success(), true);

        let output = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
            .arg("apply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("override")
            .arg("--destination")
            .arg(actual_path.to_str().unwrap())
            .arg("--source")
            .arg(template_path.to_str().unwrap())
            .arg("--source-subfolder")
            .arg(source_subfolder_1)
            .arg("-v")
            .arg("project_name=my-project")
            .ok()?;

        assert_is_same(&actual_path, &expected_path, &output)?;

        let tmp_dir_reapply = tempdir()?;
        let reapply_path = tmp_dir_reapply.path().to_path_buf();

        Command::new("cp")
            .arg("-r")
            .arg(tmp_dir_apply.path().join(".ffizer"))
            .arg(tmp_dir_reapply.path().join(".ffizer"))
            .ok()?;

        let output_reapply = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
            .arg("reapply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("override")
            .arg("--destination")
            .arg(reapply_path.to_str().unwrap())
            .ok()?;
        assert_is_same(&reapply_path, &actual_path, &output_reapply)?;
        Ok(())
    }
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
