extern crate dialoguer;
extern crate failure;
extern crate handlebars;
extern crate indicatif;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate slog;
extern crate walkdir;

#[cfg(test)]
extern crate spectral;

mod template_cfg;

use failure::format_err;
use failure::Error;
use handlebars::Handlebars;
use slog::o;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use template_cfg::TemplateCfg;
use walkdir::WalkDir;

const FILEEXT_HANDLEBARS: &'static str = ".ffizer.hbs";

#[derive(Debug, Clone)]
pub struct Ctx {
    pub logger: slog::Logger,
    pub dst_folder: PathBuf,
    pub src_uri: String,
}

impl Default for Ctx {
    fn default() -> Ctx {
        Ctx {
            logger: slog::Logger::root(slog::Discard, o!()),
            dst_folder: PathBuf::default(),
            src_uri: String::default(),
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChildPath {
    pub relative: PathBuf,
    pub base: PathBuf,
    pub is_symlink: bool,
}

impl<'a> From<&'a ChildPath> for PathBuf {
    fn from(v: &ChildPath) -> Self {
        v.base.join(&v.relative)
    }
}

// pub struct TemplateDef {
//     uri: String,
// }

// pub struct Template {
//     def: TemplateDef,
//     root_path: PathBuf,
//     input_paths: Vec<DirEntry>,
// }

pub fn process(ctx: &Ctx) -> Result<(), Error> {
    let template_base_path = as_local_path(&ctx.src_uri)?;
    let template_cfg = TemplateCfg::from_template_folder(&template_base_path)?;
    // TODO define values and ask missing
    let variables = ask_variables(&template_cfg)?;
    let input_paths = find_childpaths(template_base_path);
    let actions = plan(ctx, input_paths)?;
    if confirm_plan(&ctx, &actions)? {
        execute(ctx, &actions, &variables)
    } else {
        Ok(())
    }
}

/// list actions to execute
pub fn plan(ctx: &Ctx, src_paths: Vec<ChildPath>) -> Result<Vec<Action>, Error> {
    let mut actions = src_paths
        .into_iter()
        .map(|src_path| {
            let dst_path = compute_dst_path(ctx, &src_path).expect("TODO");
            Action {
                src_path,
                dst_path,
                operation: FileOperation::Nothing,
            }
        }).collect::<Vec<_>>();
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

fn cmp_path_for_plan(a: &Action, b: &Action) -> Ordering {
    let cmp_dst = a.dst_path.relative.cmp(&b.dst_path.relative);
    if cmp_dst == Ordering::Equal {
        a.src_path.relative.cmp(&b.src_path.relative)
    } else {
        cmp_dst
    }
}

//TODO add flag to filter display: all, changes, none
//TODO add flag to confirm: auto, always, never
fn confirm_plan(_ctx: &Ctx, actions: &Vec<Action>) -> Result<bool, std::io::Error> {
    use dialoguer::Confirmation;

    println!("Plan");
    actions.iter().for_each(|a| {
        println!("{:?}", a);
    });
    Confirmation::new("Do you want to apply plan ?").interact()
    //Ok(confirm)
}

//TODO accumulate Result (and error)
pub fn execute(
    _ctx: &Ctx,
    actions: &Vec<Action>,
    variables: &BTreeMap<String, String>,
) -> Result<(), Error> {
    use indicatif::ProgressBar;

    let pb = ProgressBar::new(actions.len() as u64);
    let handlebars = Handlebars::new();

    pb.wrap_iter(actions.iter()).for_each(|a| {
        match a.operation {
            // TODO bench performance vs create_dir (and keep create_dir_all for root aka relative is empty)
            FileOperation::MkDir => fs::create_dir_all(&PathBuf::from(&a.dst_path)).expect("TODO"),
            FileOperation::CopyRaw => {
                fs::copy(&PathBuf::from(&a.src_path), &PathBuf::from(&a.dst_path)).expect("TODO");
            }
            FileOperation::CopyRender => {
                let src = fs::read_to_string(&PathBuf::from(&a.src_path)).expect("TODO");
                let dst = fs::File::create(PathBuf::from(&a.dst_path)).expect("TODO");
                handlebars
                    .render_template_to_write(&src, variables, dst)
                    .expect("TODO");
            }
            _ => (),
        };
    });
    Ok(())
}

fn as_local_path<S>(uri: S) -> Result<PathBuf, Error>
where
    S: AsRef<str>,
{
    //TODO download / clone / pull templates if it is not local
    Ok(PathBuf::from(uri.as_ref()))
}

fn find_childpaths<P>(base: P) -> Vec<ChildPath>
where
    P: AsRef<Path>,
{
    let base = base.as_ref();
    WalkDir::new(base)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|entry| ChildPath {
            base: base.to_path_buf(),
            is_symlink: entry.path_is_symlink(),
            relative: entry
                .into_path()
                .strip_prefix(base)
                .expect("scanned child path to be under base")
                .to_path_buf(),
        }).collect::<Vec<_>>()
}

fn compute_dst_path(ctx: &Ctx, src: &ChildPath) -> Result<ChildPath, Error> {
    let relative = if is_ffizer_handlebars(&src.relative) {
        let mut file_name = src
            .relative
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or(format_err!("faile to extract file_name"))?;
        file_name = file_name
            .get(..file_name.len() - FILEEXT_HANDLEBARS.len())
            .ok_or(format_err!(
                "failed to remove {} from file_name",
                FILEEXT_HANDLEBARS
            ))?;
        src.relative.with_file_name(file_name)
    } else {
        src.relative.clone()
    };

    Ok(ChildPath {
        base: ctx.dst_folder.clone(),
        relative,
        is_symlink: src.is_symlink,
    })
}

fn select_operation(
    _ctx: &Ctx,
    src_path: &ChildPath,
    dst_path: &ChildPath,
    _actions: &Vec<Action>,
) -> FileOperation {
    let src_full_path = PathBuf::from(src_path);
    let dest_full_path = PathBuf::from(dst_path);
    if dest_full_path.exists()
    /* || actions.contains(dst_path) propably the last */
    {
        FileOperation::Keep
    } else if src_full_path.is_dir() {
        FileOperation::MkDir
    } else if is_ffizer_handlebars(&src_full_path) {
        FileOperation::CopyRender
    } else {
        FileOperation::CopyRaw
    }
}

fn is_ffizer_handlebars(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|str| str.ends_with(FILEEXT_HANDLEBARS))
        .unwrap_or(false)
}

fn ask_variables(cfg: &TemplateCfg) -> Result<BTreeMap<String, String>, Error> {
    use dialoguer::Input;
    let mut variables = BTreeMap::new();
    for variable in cfg.variables.iter() {
        let name = variable.name.clone();
        //TODO use variable.ask : let ask = &(variable.ask).unwrap_or(name);
        let mut input = Input::new(&name);
        if let Some(default_value) = &variable.default_value {
            input.default(default_value);
        }
        // TODO manage error
        let value = input.interact().expect("valid interaction");
        variables.insert(name, value);
    }
    Ok(variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn test_compute_dst_path_asis() {
        let ctx = Ctx {
            dst_folder: PathBuf::from("test/dst"),
            ..Default::default()
        };
        let src = ChildPath {
            relative: PathBuf::from("hello/sample.txt"),
            base: PathBuf::from("test/src"),
            is_symlink: false,
        };
        let expected = ChildPath {
            relative: PathBuf::from("hello/sample.txt"),
            base: ctx.dst_folder.clone(),
            is_symlink: false,
        };
        let actual = compute_dst_path(&ctx, &src).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn test_compute_dst_path_ffizer_handlebars() {
        let ctx = Ctx {
            dst_folder: PathBuf::from("test/dst"),
            ..Default::default()
        };
        let src = ChildPath {
            relative: PathBuf::from("hello/sample.txt.ffizer.hbs"),
            base: PathBuf::from("test/src"),
            is_symlink: false,
        };
        let expected = ChildPath {
            relative: PathBuf::from("hello/sample.txt"),
            base: ctx.dst_folder.clone(),
            is_symlink: false,
        };
        let actual = compute_dst_path(&ctx, &src).unwrap();
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
    fn test_is_ffizer_handlebars() {
        assert_that!(is_ffizer_handlebars(&PathBuf::from("foo.hbs"))).is_false();
        assert_that!(is_ffizer_handlebars(&PathBuf::from("foo.ffizer.hbs/bar"))).is_false();
        assert_that!(is_ffizer_handlebars(&PathBuf::from("foo_ffizer.hbs"))).is_false();
        assert_that!(is_ffizer_handlebars(&PathBuf::from("fooffizer.hbs"))).is_false();

        assert_that!(is_ffizer_handlebars(&PathBuf::from("foo.ffizer.hbs"))).is_true();
        assert_that!(is_ffizer_handlebars(&PathBuf::from("bar/foo.ffizer.hbs"))).is_true();
    }
}
