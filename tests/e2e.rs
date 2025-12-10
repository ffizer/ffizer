use assert_cmd::cargo::cargo_bin_cmd;
use ffizer::PathPattern;
use ffizer::tools::dir_diff_list;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};
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
    cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
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
    for diff in diffs {
        match diff.difference {
            dir_diff_list::Difference::StringContent { expect, actual } => {
                println!("StringContent mismatch on {:?}", diff.relative_path);
                assert_eq!(actual, expect)
            }
            _ => assert_eq!(format!("{:?}", diff), ""),
        }
    }

    assert_eq!(output.status.success(), true);
    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

mod test_reapply {
    use super::*;
    use rstest::*;
    use similar_asserts::assert_eq;

    #[rstest]
    fn single_template_inplace() -> Result<(), Box<dyn Error>> {
        let source_subfolder = "template_1";
        let tmp_dir = tempdir()?;
        let template_path = PathBuf::from("./tests/data");
        let expected_path =
            PathBuf::from("./tests/data/template_1/.ffizer.samples.d/my-project.expected");
        let apply_result_path = tmp_dir.path().to_path_buf();

        let output = cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
            .arg("apply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("keep")
            .arg("--destination")
            .arg(apply_result_path.to_str().unwrap())
            .arg("--source")
            .arg(template_path.to_str().unwrap())
            .arg("--source-subfolder")
            .arg(source_subfolder)
            .ok()?;
        assert_is_same(&apply_result_path, &expected_path, &output)?;

        let output_reapply = cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
            .arg("reapply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("keep")
            .arg("--destination")
            .arg(apply_result_path.to_str().unwrap())
            .ok()?;
        assert_is_same(&apply_result_path, &expected_path, &output_reapply)?;

        Ok(())
    }

    #[rstest]
    fn single_template_moved() -> Result<(), Box<dyn Error>> {
        let source_subfolder = "template_1";
        let tmp_dir_apply = tempdir()?;
        let template_path = PathBuf::from("./tests/data");
        let expected_path =
            PathBuf::from("./tests/data/template_1/.ffizer.samples.d/my-project.expected");
        let apply_result_path = tmp_dir_apply.path().to_path_buf();

        let output = cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
            .arg("apply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("keep")
            .arg("--destination")
            .arg(apply_result_path.to_str().unwrap())
            .arg("--source")
            .arg(template_path.to_str().unwrap())
            .arg("--source-subfolder")
            .arg(source_subfolder)
            .ok()?;
        assert_is_same(&apply_result_path, &expected_path, &output)?;

        let tmp_dir_reapply = tempdir()?;
        let reapply_result_path = tmp_dir_reapply.path().to_path_buf();

        copy_dir_all(
            tmp_dir_apply.path().join(".ffizer"),
            tmp_dir_reapply.path().join(".ffizer"),
        )?;

        let output_reapply = cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
            .arg("reapply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("keep")
            .arg("--destination")
            .arg(reapply_result_path.to_str().unwrap())
            .ok()?;
        assert_is_same(&reapply_result_path, &expected_path, &output_reapply)?;

        Ok(())
    }

    //FIXME apply 2 templates doesn't generate the same output than appling
    // a compose (import) of the 2 templates (look at the transitive)
    // eg
    // ```
    //     Diff < left / right > :
    // <[
    // <    EntryDiff {
    // <        expect_base_path: "/tmp/.tmpS9qvPz",
    // <        actual_base_path: "/tmp/.tmppUDqqE",
    // <        relative_path: "file_5.txt",
    // <        difference: StringContent {
    // <            expect: "content from template_1 before\ncontent from template_1_1 before\ncontent from template_2 before\n\ncontent from template_2 after\n\ncontent from template_1_1 after\n\ncontent from template_1 after\n",
    // <            actual: "content from template_1 before\ncontent from template_2 before\ncontent from template_1_1 before\n\ncontent from template_1_1 after\n\ncontent from template_2 after\n\ncontent from template_1 after\n",
    // <        },
    // <    },
    // <]
    // ```
    // #[rstest]
    fn _multi_template_moved() -> Result<(), Box<dyn Error>> {
        let tmp_dir_apply = tempdir()?;
        let template_path = PathBuf::from("./tests/data/4compose");
        let apply_result_path = tmp_dir_apply.path().to_path_buf();

        for source_subfolder in ["template_2", "template_1"] {
            cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
                .arg("apply")
                .arg("--no-interaction")
                .arg("--confirm")
                .arg("never")
                .arg("--update-mode")
                .arg("override")
                .arg("--destination")
                .arg(apply_result_path.to_str().unwrap())
                .arg("--source")
                .arg(template_path.to_str().unwrap())
                .arg("--source-subfolder")
                .arg(source_subfolder)
                .arg("-v")
                .arg("project_name=my-project")
                .ok()?;
        }

        let tmp_dir_reapply = tempdir()?;
        let reapply_result_path = tmp_dir_reapply.path().to_path_buf();

        copy_dir_all(
            apply_result_path.join(".ffizer"),
            reapply_result_path.join(".ffizer"),
        )?;

        let output_reapply = cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
            .arg("reapply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("override")
            .arg("--destination")
            .arg(reapply_result_path.to_str().unwrap())
            .ok()?;
        assert_is_same(&reapply_result_path, &apply_result_path, &output_reapply)?;
        Ok(())
    }

    //FIXME apply 2 templates doesn't generate the same output than appling
    // a compose (import) of the 2 templates (look at the transitive)
    // eg
    // ```
    // Diff < left / right > :
    // <[
    // <    EntryDiff {
    // <        expect_base_path: "./tests/data/reapply/expected",
    // <        actual_base_path: "/tmp/.tmpQgLE8n",
    // <        relative_path: "file_5.txt",
    // <        difference: StringContent {
    // <            expect: "content from template_1 before\ncontent from template_1_1 before\ncontent from template_2 before\n\ncontent from template_2 after\n\ncontent from template_1_1 after\n\ncontent from template_1 after\n",
    // <            actual: "content from template_1 before\ncontent from template_2 before\ncontent from template_1_1 before\ncontent from template_1 before\ncontent from template_1_1 before\ncontent from template_2 before\n\ncontent from template_2 after\n\ncontent from template_1_1 after\n\ncontent from template_1 after\n\ncontent from template_1_1 after\n\ncontent from template_2 after\n\ncontent from template_1 after\n",
    // <        },
    // <    },
    // <]
    // ```
    // #[rstest]
    fn _multi_template_inplace() -> Result<(), Box<dyn Error>> {
        let tmp_dir_apply = tempdir()?;
        let template_path = PathBuf::from("./tests/data/4compose");
        let expected_path = PathBuf::from("./tests/data/reapply/expected");
        let apply_result_path = tmp_dir_apply.path().to_path_buf();

        for source_subfolder in ["template_2", "template_1"] {
            let output = cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
                .arg("apply")
                .arg("--no-interaction")
                .arg("--confirm")
                .arg("never")
                .arg("--update-mode")
                .arg("override")
                .arg("--destination")
                .arg(apply_result_path.to_str().unwrap())
                .arg("--source")
                .arg(template_path.to_str().unwrap())
                .arg("--source-subfolder")
                .arg(source_subfolder)
                .arg("-v")
                .arg("project_name=my-project")
                .ok()?;

            assert_eq!(output.status.success(), true);
        }

        let output_reapply = cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
            .arg("reapply")
            .arg("--no-interaction")
            .arg("--confirm")
            .arg("never")
            .arg("--update-mode")
            .arg("override")
            .arg("--destination")
            .arg(apply_result_path.to_str().unwrap())
            .ok()?;
        assert_is_same(&apply_result_path, &expected_path, &output_reapply)?;
        Ok(())
    }
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_master() -> Result<(), Box<dyn Error>> {
    cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
        .arg("test-samples")
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .ok()?;
    Ok(())
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_commithash() -> Result<(), Box<dyn Error>> {
    cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
        .arg("test-samples")
        .arg("--source")
        .arg("https://github.com/ffizer/template_sample.git")
        .arg("--rev")
        .arg("3ab3bc67b5fab58ceecc031f7ed0eb29c0e0fff8") // Devskim: ignore DS173237
        .ok()?;
    Ok(())
}

#[cfg(feature = "test_remote")]
#[test]
fn test_1_remote_tag() -> Result<(), Box<dyn Error>> {
    cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
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

    cargo_bin_cmd!(env!("CARGO_PKG_NAME"))
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
                .and(predicate::str::contains("reason: InvalidSyntax")),
        )
        .failure();
    Ok(())
}
