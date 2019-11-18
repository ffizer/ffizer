#[macro_use]
extern crate serde;

mod cli_opt;
mod error;
mod files;
mod git;
mod graph;
mod path_pattern;
mod source_loc;
mod source_uri;
mod template_cfg;
mod template_composite;
mod transform_values;
mod tree;
mod ui;

pub use crate::cli_opt::*;
pub use crate::error::*;
pub use crate::source_loc::SourceLoc;

use crate::files::is_ffizer_handlebars;
use crate::files::ChildPath;
use crate::template_composite::TemplateComposite;
use handlebars_misc_helpers::new_hbs;
use slog::{debug, o};
use snafu::ResultExt;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub type Variables = BTreeMap<String, String>;

#[derive(Debug, Clone)]
pub struct Ctx {
    pub logger: slog::Logger,
    pub cmd_opt: ApplyOpts,
}

impl Default for Ctx {
    fn default() -> Ctx {
        Ctx {
            logger: slog::Logger::root(slog::Discard, o!()),
            cmd_opt: ApplyOpts::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileOperation {
    Nothing,
    Ignore,
    MkDir,
    AddFile,
    UpdateFile,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub src_path: ChildPath,
    pub dst_path: ChildPath,
    // template: TemplateDef,
    pub operation: FileOperation,
}

pub fn process(ctx: &Ctx) -> Result<()> {
    let variables_from_cli = extract_variables(&ctx)?;
    let template_composite = TemplateComposite::from_src(
        &ctx,
        &variables_from_cli,
        ctx.cmd_opt.offline,
        &ctx.cmd_opt.src,
    )?;
    let variables = ui::ask_variables(&ctx, &template_composite.variables(), variables_from_cli)?;
    // update cfg with variables defined by user (use to update ignore)
    //TODO template_cfg = render_cfg(&ctx, &template_cfg, &variables, true)?;
    let input_paths = template_composite.find_childpaths()?;
    let actions = plan(ctx, input_paths, &variables)?;
    if ui::confirm_plan(&ctx, &actions)? {
        execute(ctx, &actions, &variables)
    } else {
        Ok(())
    }
}

pub fn extract_variables(ctx: &Ctx) -> Result<Variables> {
    let mut variables = Variables::new();
    variables.insert(
        "ffizer_dst_folder".to_owned(),
        ctx.cmd_opt
            .dst_folder
            .to_str()
            .expect("dst_folder to converted via to_str")
            .to_owned(),
    );
    variables.insert("ffizer_src_uri".to_owned(), ctx.cmd_opt.src.uri.raw.clone());
    variables.insert("ffizer_src_rev".to_owned(), ctx.cmd_opt.src.rev.clone());
    Ok(variables)
}

/// list actions to execute
fn plan(ctx: &Ctx, src_paths: Vec<ChildPath>, variables: &Variables) -> Result<Vec<Action>> {
    let mut actions = src_paths
        .into_iter()
        .map(|src_path| {
            compute_dst_path(ctx, &src_path, variables).map(|dst_path| Action {
                src_path,
                dst_path,
                operation: FileOperation::Nothing,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    // TODO sort input_paths by priority (*.ffizer(.*) first, alphabetical)
    actions.sort_by(cmp_path_for_plan);
    let actions_count = actions.len();
    actions = actions
        .into_iter()
        .fold(Vec::with_capacity(actions_count), |mut acc, e| {
            let operation = select_operation(ctx, &e.src_path, &e.dst_path, &acc);
            acc.push(Action { operation, ..e });
            acc
        });
    Ok(actions)
}

// TODO add test
// TODO add priority for generated file name / folder name
// TODO document priority (via test ?)
fn cmp_path_for_plan(a: &Action, b: &Action) -> Ordering {
    let cmp_dst = a.dst_path.relative.cmp(&b.dst_path.relative);
    if cmp_dst != Ordering::Equal {
        cmp_dst
    } else if a
        .src_path
        .relative
        .to_str()
        .map(|s| s.contains("{{"))
        .unwrap_or(false)
    {
        Ordering::Greater
    } else if is_ffizer_handlebars(&a.src_path.relative) {
        Ordering::Less
    } else if is_ffizer_handlebars(&b.src_path.relative) {
        Ordering::Greater
    } else {
        a.src_path.relative.cmp(&b.src_path.relative)
    }
}

//TODO accumulate Result (and error)
fn execute(ctx: &Ctx, actions: &[Action], variables: &Variables) -> Result<()> {
    use indicatif::ProgressBar;

    let pb = ProgressBar::new(actions.len() as u64);
    let handlebars = new_hbs();
    debug!(ctx.logger, "execute"; "variables" => ?&variables);

    for a in pb.wrap_iter(actions.iter()) {
        match a.operation {
            FileOperation::Nothing => (),
            FileOperation::Ignore => (),
            // TODO bench performance vs create_dir (and keep create_dir_all for root aka relative is empty)
            FileOperation::MkDir => {
                let path = PathBuf::from(&a.dst_path);
                fs::create_dir_all(&path).context(CreateFolder { path })?
            }
            FileOperation::AddFile => {
                mk_file_on_action(&handlebars, variables, &a, "").map(|_| ())?
            }
            FileOperation::UpdateFile => {
                //TODO what to do if .LOCAL, .REMOTE already exist ?
                let (local, remote) = mk_file_on_action(&handlebars, variables, &a, ".REMOTE")?;
                let local_digest = md5::compute(fs::read(&local).context(ReadFile {
                    path: local.clone(),
                })?);
                let remote_digest = md5::compute(fs::read(&remote).context(ReadFile {
                    path: remote.clone(),
                })?);
                if local_digest == remote_digest {
                    fs::remove_file(&remote).context(RemoveFile {
                        path: remote.clone(),
                    })?
                } else {
                    update_file(&local, &remote, &ctx.cmd_opt.update_mode)?
                }
            }
        }
    }
    Ok(())
}

fn mk_file_on_action(
    handlebars: &handlebars::Handlebars,
    variables: &Variables,
    a: &Action,
    dest_suffix_ext: &str,
) -> Result<(PathBuf, PathBuf)> {
    let src_full_path = PathBuf::from(&a.src_path);
    let dest_full_path_target = PathBuf::from(&a.dst_path);
    mk_file(
        handlebars,
        variables,
        src_full_path,
        dest_full_path_target,
        dest_suffix_ext,
    )
}

fn mk_file<P>(
    handlebars: &handlebars::Handlebars,
    variables: &Variables,
    src_full_path: P,
    dest_full_path_target: P,
    dest_suffix_ext: &str,
) -> Result<(PathBuf, PathBuf)>
where
    P: AsRef<std::path::Path>,
{
    let src_full_path = src_full_path.as_ref();
    let dest_full_path_target = dest_full_path_target.as_ref();
    let dest_full_path = files::add_suffix(dest_full_path_target, dest_suffix_ext)?;
    if !is_ffizer_handlebars(src_full_path) {
        fs::copy(&src_full_path, &dest_full_path).context(CopyFile {
            src: src_full_path.clone(),
            dst: dest_full_path.clone(),
        })?;
    } else {
        let src = fs::read_to_string(&src_full_path).context(ReadFile {
            path: src_full_path.clone(),
        })?;
        let dst = fs::File::create(&dest_full_path).context(CreateFile {
            path: dest_full_path.clone(),
        })?;
        handlebars
            .render_template_to_write(&src, variables, dst)
            .context(crate::Handlebars {
                when: format!("define content for '{:?}'", dest_full_path),
                template: src.clone(),
            })?;
    }
    Ok((PathBuf::from(&dest_full_path_target), dest_full_path))
}

fn update_file<P>(local: P, remote: P, mode_init: &UpdateMode) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    let mut mode = mode_init.clone();
    let remote = remote.as_ref();
    let local = local.as_ref();
    loop {
        match mode {
            UpdateMode::Ask => {
                mode = ui::ask_update_mode(&local)?;
            }
            UpdateMode::ShowDiff => {
                // show diff (then re-ask)
                ui::show_difference(&local, &remote)?;
                mode = UpdateMode::Ask;
            }
            UpdateMode::Override => {
                fs::remove_file(&local).context(RemoveFile {
                    path: local.clone(),
                })?;
                fs::rename(&remote, &local).context(RenameFile {
                    src: remote.clone(),
                    dst: local.clone(),
                })?;
                break;
            }
            UpdateMode::Keep => {
                fs::remove_file(&remote).context(RemoveFile {
                    path: remote.clone(),
                })?;
                break;
            }
            UpdateMode::UpdateAsRemote => {
                // store generated as .REMOTE
                // nothing todo
                break;
            }
            UpdateMode::CurrentAsLocal => {
                // backup existing as .LOCAL
                let new_local = files::add_suffix(&local, ".LOCAL")?;
                fs::rename(&local, &new_local).context(RenameFile {
                    src: local.clone(),
                    dst: new_local.clone(),
                })?;
                fs::rename(&remote, &local).context(RenameFile {
                    src: remote.clone(),
                    dst: local.clone(),
                })?;
                break;
            } // UpdateMode::Merge => {
              //     // merge tool (if defined in gitconfig)
              //     mode = UpdateMode::Ask;
              //     // break;
              // }
        }
    }
    Ok(())
}

//TODO optimise / bench to avoid creation and rendering of path handlebars
fn compute_dst_path(ctx: &Ctx, src: &ChildPath, variables: &Variables) -> Result<ChildPath> {
    let rendered_relative = src
        .relative
        .to_str()
        .ok_or(Error::Any {
            msg: "failed to stringify path".to_owned(),
        })
        .and_then(|s| {
            let handlebars = new_hbs();
            let p = handlebars
                .render_template(&s, variables)
                .context(crate::Handlebars {
                    when: format!("define path for '{:?}'", src),
                    template: s,
                })?;
            Ok(PathBuf::from(p))
        })?;
    let relative = files::remove_special_suffix(&rendered_relative)?;

    Ok(ChildPath {
        base: ctx.cmd_opt.dst_folder.clone(),
        relative,
        is_symlink: src.is_symlink,
    })
}

fn select_operation(
    _ctx: &Ctx,
    src_path: &ChildPath,
    dst_path: &ChildPath,
    actions: &[Action],
) -> FileOperation {
    let src_full_path = PathBuf::from(src_path);
    let dest_full_path = PathBuf::from(dst_path);
    if dest_full_path.exists()
        || actions
            .iter()
            .any(|a| a.dst_path.relative == dst_path.relative)
    {
        if dest_full_path.is_dir() {
            FileOperation::Nothing
        } else {
            FileOperation::UpdateFile
        }
    } else if src_full_path.is_dir() {
        FileOperation::MkDir
    } else {
        FileOperation::AddFile
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub use crate::cli_opt::*;
    use spectral::prelude::*;
    use tempfile::TempDir;

    #[test]
    fn test_cmp_path_for_plan() {
        let a = Action {
            src_path: ChildPath {
                relative: PathBuf::from("file_2.txt"),
                base: PathBuf::from("./tests/test_1/template"),
                is_symlink: false,
            },
            dst_path: ChildPath {
                relative: PathBuf::from("file_2.txt"),
                base: PathBuf::from("/tmp/.tmpYPoYTW"),
                is_symlink: false,
            },
            operation: FileOperation::Nothing,
        };
        let b = Action {
            src_path: ChildPath {
                relative: PathBuf::from("file_2.txt.ffizer.hbs"),
                base: PathBuf::from("./tests/test_1/template"),
                is_symlink: false,
            },
            dst_path: ChildPath {
                relative: PathBuf::from("file_2.txt"),
                base: PathBuf::from("/tmp/.tmpYPoYTW"),
                is_symlink: false,
            },
            operation: FileOperation::Nothing,
        };
        assert_that!(cmp_path_for_plan(&a, &b)).is_equal_to(&Ordering::Greater);
        assert_that!(cmp_path_for_plan(&b, &a)).is_equal_to(&Ordering::Less);
    }

    #[test]
    fn test_compute_dst_path_asis() {
        let ctx = Ctx {
            cmd_opt: ApplyOpts {
                dst_folder: PathBuf::from("test/dst"),
                ..Default::default()
            },
            ..Default::default()
        };
        let variables = BTreeMap::new();
        let src = ChildPath {
            relative: PathBuf::from("hello/sample.txt"),
            base: PathBuf::from("test/src"),
            is_symlink: false,
        };
        let expected = ChildPath {
            relative: PathBuf::from("hello/sample.txt"),
            base: ctx.cmd_opt.dst_folder.clone(),
            is_symlink: false,
        };
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn test_compute_dst_path_ffizer_handlebars() {
        let ctx = Ctx {
            cmd_opt: ApplyOpts {
                dst_folder: PathBuf::from("test/dst"),
                ..Default::default()
            },
            ..Default::default()
        };
        let variables = BTreeMap::new();

        let src = ChildPath {
            relative: PathBuf::from("hello/sample.txt.ffizer.hbs"),
            base: PathBuf::from("test/src"),
            is_symlink: false,
        };
        let expected = ChildPath {
            relative: PathBuf::from("hello/sample.txt"),
            base: ctx.cmd_opt.dst_folder.clone(),
            is_symlink: false,
        };
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn test_compute_dst_path_rendered_filename() {
        let ctx = Ctx {
            cmd_opt: ApplyOpts {
                dst_folder: PathBuf::from("test/dst"),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut variables = BTreeMap::new();
        variables.insert("prj".to_owned(), "myprj".to_owned());

        let src = ChildPath {
            relative: PathBuf::from("hello/{{ prj }}.txt"),
            base: PathBuf::from("test/src"),
            is_symlink: false,
        };
        let expected = ChildPath {
            relative: PathBuf::from("hello/myprj.txt"),
            base: ctx.cmd_opt.dst_folder.clone(),
            is_symlink: false,
        };
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn test_compute_dst_path_rendered_folder() {
        let ctx = Ctx {
            cmd_opt: ApplyOpts {
                dst_folder: PathBuf::from("test/dst"),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut variables = BTreeMap::new();
        variables.insert("prj".to_owned(), "myprj".to_owned());

        let src = ChildPath {
            relative: PathBuf::from("hello/{{ prj }}/sample.txt"),
            base: PathBuf::from("test/src"),
            is_symlink: false,
        };
        let expected = ChildPath {
            relative: PathBuf::from("hello/myprj/sample.txt"),
            base: ctx.cmd_opt.dst_folder.clone(),
            is_symlink: false,
        };
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn test_path_extension_extraction() {
        use std::ffi::OsStr;

        assert_that!(PathBuf::from("foo.ext1").extension()).is_equal_to(&Some(OsStr::new("ext1")));
        assert_that!(PathBuf::from("foo.ext2.ext1").extension())
            .is_equal_to(&Some(OsStr::new("ext1")));
    }

    #[test]
    fn test_mk_file_by_copy() {
        // Create a directory inside of `std::env::temp_dir()`
        let tmp_dir = TempDir::new().expect("create a temp dir");

        let src_path = tmp_dir.path().join("src.txt");
        fs::write(&src_path, "src {{ prj }}").expect("create local file");

        let dst_path = tmp_dir.path().join("dst.txt");
        let handlebars = new_hbs();
        let mut variables = BTreeMap::new();
        variables.insert("prj".to_owned(), "myprj".to_owned());

        mk_file(&handlebars, &variables, &src_path, &dst_path, "").expect("mk_file is ok");
        assert_that!(&dst_path).exists();
        assert_that!(fs::read_to_string(&dst_path).unwrap())
            .is_equal_to("src {{ prj }}".to_owned());
    }

    #[test]
    fn test_mk_file_by_render() {
        // Create a directory inside of `std::env::temp_dir()`
        let tmp_dir = TempDir::new().expect("create a temp dir");

        let src_path = tmp_dir.path().join("src.txt.ffizer.hbs");
        fs::write(&src_path, "src {{ prj }}").expect("create local file");

        let dst_path = tmp_dir.path().join("dst.txt");
        let handlebars = new_hbs();
        let mut variables = BTreeMap::new();
        variables.insert("prj".to_owned(), "myprj".to_owned());

        mk_file(&handlebars, &variables, &src_path, &dst_path, "").expect("mk_file is ok");
        assert_that!(&dst_path).exists();
        assert_that!(fs::read_to_string(&dst_path).unwrap()).is_equal_to("src myprj".to_owned());
    }

    const CONTENT_LOCAL: &str = "local";
    const CONTENT_REMOTE: &str = "remote";

    fn setup_for_test_update() -> (TempDir, PathBuf, PathBuf) {
        // Create a directory inside of `std::env::temp_dir()`
        let tmp_dir = TempDir::new().expect("create a temp dir");

        let local_path = tmp_dir.path().join("file.txt");
        fs::write(&local_path, CONTENT_LOCAL).expect("create local file");

        let remote_path = tmp_dir.path().join("file.txt.REMOTE");
        fs::write(&remote_path, CONTENT_REMOTE).expect("create remote file");

        (tmp_dir, local_path, remote_path)
    }

    #[test]
    fn test_update_file_override() {
        // grab _tmp_dir, because Drop will delete it and its files
        let (_tmp_dir, local_path, remote_path) = setup_for_test_update();
        update_file(&local_path, &remote_path, &UpdateMode::Override)
            .expect("update without error");
        assert_that!(&local_path).exists();
        assert_that!(fs::read_to_string(&local_path).unwrap())
            .is_equal_to(CONTENT_REMOTE.to_owned());
        assert_that!(&remote_path).does_not_exist();
    }

    #[test]
    fn test_update_file_keep() {
        // grab _tmp_dir, because Drop will delete it and its files
        let (_tmp_dir, local_path, remote_path) = setup_for_test_update();
        update_file(&local_path, &remote_path, &UpdateMode::Keep).expect("update without error");
        assert_that!(&local_path).exists();
        assert_that!(fs::read_to_string(&local_path).unwrap())
            .is_equal_to(CONTENT_LOCAL.to_owned());
        assert_that!(&remote_path).does_not_exist();
    }

    #[test]
    fn test_update_file_update_as_remote() {
        // grab _tmp_dir, because Drop will delete it and its files
        let (_tmp_dir, local_path, remote_path) = setup_for_test_update();
        update_file(&local_path, &remote_path, &UpdateMode::UpdateAsRemote)
            .expect("update without error");
        assert_that!(&local_path).exists();
        assert_that!(fs::read_to_string(&local_path).unwrap())
            .is_equal_to(CONTENT_LOCAL.to_owned());
        assert_that!(&remote_path).exists();
        assert_that!(fs::read_to_string(&remote_path).unwrap())
            .is_equal_to(CONTENT_REMOTE.to_owned());
    }

    #[test]
    fn test_update_file_current_as_local() {
        // grab _tmp_dir, because Drop will delete it and its files
        let (_tmp_dir, local_path, remote_path) = setup_for_test_update();
        update_file(&local_path, &remote_path, &UpdateMode::CurrentAsLocal)
            .expect("update without error");
        assert_that!(&local_path).exists();
        assert_that!(fs::read_to_string(&local_path).unwrap())
            .is_equal_to(CONTENT_REMOTE.to_owned());
        assert_that!(&remote_path).does_not_exist();
        let dot_local_path = files::add_suffix(&local_path, ".LOCAL").unwrap();
        assert_that!(&dot_local_path).exists();
        assert_that!(fs::read_to_string(&dot_local_path).unwrap())
            .is_equal_to(CONTENT_LOCAL.to_owned());
    }

    // #[test]
    // fn test_update_file_show_diff() {
    //     // grab _tmp_dir, because Drop will delete it and its files
    //     let (_tmp_dir, local_path, remote_path) = setup_for_test_update();
    //     update_file(&local_path, &remote_path, &UpdateMode::ShowDiff)
    //         .expect("update without error");
    // }
}
