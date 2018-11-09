#[macro_use]
pub extern crate slog;
extern crate walkdir;

use std::cmp::Ordering;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

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

pub fn process(ctx: &Ctx) -> Result<(), Box<Error>> {
    // TODO define values and ask missing
    let template_base_path = as_local_path(&ctx.src_uri)?;
    let input_paths = find_childpaths(template_base_path);
    let actions = plan(ctx, input_paths)?;
    //TODO display actions ask for confirmation
    execute(ctx, &actions)
}

/// list actions to execute
pub fn plan(ctx: &Ctx, src_paths: Vec<ChildPath>) -> Result<Vec<Action>, Box<Error>> {
    let mut actions = src_paths
        .into_iter()
        .map(|src_path| {
            let dst_path = compute_dst_path(ctx, &src_path);
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

//TODO accumulate Result (and error)
pub fn execute(_ctx: &Ctx, actions: &Vec<Action>) -> Result<(), Box<Error>> {
    actions.iter().for_each(|a| {
        println!("TODO {:?}", a);
        match a.operation {
            // TODO bench performance vs create_dir (and keep create_dir_all for root aka relative is empty)
            FileOperation::MkDir => fs::create_dir_all(&PathBuf::from(&a.dst_path)).expect("TODO"),
            FileOperation::CopyRaw => {
                fs::copy(&PathBuf::from(&a.src_path), &PathBuf::from(&a.dst_path)).expect("TODO");
            }
            _ => (),
        };
    });

    Ok(())
}

fn as_local_path<S>(uri: S) -> Result<PathBuf, Box<Error>>
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

fn compute_dst_path(ctx: &Ctx, src: &ChildPath) -> ChildPath {
    ChildPath {
        base: ctx.dst_folder.clone(),
        relative: src.relative.clone(),
        is_symlink: src.is_symlink,
    }
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
    } else {
        FileOperation::CopyRaw
    }
}

#[cfg(test)]
mod tests {
    extern crate spectral;

    use super::*;
    use tests::spectral::prelude::*;

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
        let actual = compute_dst_path(&ctx, &src);
        assert_that!(&actual).is_equal_to(&expected);
    }
}
