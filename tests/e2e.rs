//extern crate dir_diff;
extern crate failure;
extern crate ffizer;
extern crate tempfile;

use failure::Error;
use std::fs;
use std::path::PathBuf;
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

    let ctx = ffizer::Ctx {
        cmd_opt: ffizer::CmdOpt {
            dst_folder: actual_path.clone(),
            src_uri: template_path.to_str().unwrap().to_owned(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(ffizer::process(&ctx).is_ok());

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

    let ctx = ffizer::Ctx {
        cmd_opt: ffizer::CmdOpt {
            dst_folder: actual_path.clone(),
            src_uri: template_path.to_str().unwrap().to_owned(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(ffizer::process(&ctx).is_ok());

    dir_diff::is_same(&actual_path, &expected_path)?;
    Ok(())
}
