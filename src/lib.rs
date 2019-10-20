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
mod ui;
mod tree;

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
    Keep,
    MkDir,
    CopyRaw,
    CopyRender,
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
            // TODO bench performance vs create_dir (and keep create_dir_all for root aka relative is empty)
            FileOperation::MkDir => {
                fs::create_dir_all(&PathBuf::from(&a.dst_path)).context(Io {})?
            }
            FileOperation::CopyRaw => {
                fs::copy(&PathBuf::from(&a.src_path), &PathBuf::from(&a.dst_path))
                    .context(Io {})?;
            }
            FileOperation::CopyRender => {
                let src = fs::read_to_string(&PathBuf::from(&a.src_path)).context(Io {})?;
                let dst = fs::File::create(PathBuf::from(&a.dst_path)).context(Io {})?;
                handlebars
                    .render_template_to_write(&src, variables, dst)
                    .context(crate::Handlebars {
                        when: format!("define content for '{:?}'", &a),
                        template: src.clone(),
                    })?;
            }
            _ => (),
        };
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
    // optim: propably the last
    {
        FileOperation::Keep
    // } else if src_path
    //     .relative
    //     .to_str()
    //     .map(|s| cfg.ignores.iter().any(|f| f.is_match(s)))
    //     .unwrap_or(false)
    // {
    //     FileOperation::Ignore
    } else if src_full_path.is_dir() {
        FileOperation::MkDir
    } else if is_ffizer_handlebars(&src_full_path) {
        FileOperation::CopyRender
    } else {
        FileOperation::CopyRaw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

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
}
