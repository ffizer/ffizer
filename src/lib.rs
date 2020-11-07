#[macro_use]
extern crate serde;

pub mod error;
pub mod tools;

mod cfg;
mod cli_opt;
mod files;
mod git;
mod graph;
mod path_pattern;
mod scripts;
mod source_file;
mod source_loc;
mod source_uri;
mod ui;
mod variable_def;
mod variables;

pub use crate::cfg::provide_json_schema;
pub use crate::cli_opt::*;
pub use crate::source_loc::SourceLoc;

use crate::cfg::{render_composite, TemplateComposite};
use crate::error::*;
use crate::files::ChildPath;
use crate::source_file::{SourceFile, SourceFileMetadata};
use crate::variables::Variables;
use handlebars_misc_helpers::new_hbs;
use slog::{debug, o, warn};
use std::fs;
use std::path::PathBuf;

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
    pub src: Vec<SourceFile>,
    pub dst_path: ChildPath,
    // template: TemplateDef,
    pub operation: FileOperation,
}

pub fn process(ctx: &Ctx) -> Result<()> {
    debug!(ctx.logger, "extracting variables from cli");
    let variables_from_cli = extract_variables(&ctx)?;
    debug!(ctx.logger, "compositing templates");
    let mut template_composite = TemplateComposite::from_src(
        &ctx,
        &variables_from_cli,
        ctx.cmd_opt.offline,
        &ctx.cmd_opt.src,
    )?;
    debug!(ctx.logger, "asking variables");
    let variables = ui::ask_variables(
        &ctx,
        &template_composite.find_variabledefs()?,
        variables_from_cli,
    )?;
    // update cfg(s) with variables defined by user (use to update ignore, scripts,...)
    template_composite = render_composite(&ctx, &template_composite, &variables, true)?;
    debug!(ctx.logger, "listing files from templates");
    let source_files = template_composite.find_sourcefiles()?;
    debug!(ctx.logger, "defining plan of rendering");
    let actions = plan(ctx, source_files, &variables)?;
    if ui::confirm_plan(&ctx, &actions)? {
        debug!(ctx.logger, "executing plan of rendering");
        execute(ctx, &actions, &variables)?;
        debug!(ctx.logger, "running scripts");
        run_scripts(ctx, &template_composite)?;
    }
    Ok(())
}

fn do_in_folder<F, R>(folder: &PathBuf, f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    fs::create_dir_all(&folder).map_err(|source| Error::CreateFolder {
        path: folder.clone(),
        source,
    })?;
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(&folder)?;
    // let res = apply_plan(&ctx, &actions, &variables, &template_composite);
    let res = f();
    let _ = std::env::set_current_dir(&current_dir)?;
    res
}

pub fn extract_variables(ctx: &Ctx) -> Result<Variables> {
    let mut variables = Variables::default();
    variables.insert(
        "ffizer_dst_folder",
        ctx.cmd_opt
            .dst_folder
            .to_str()
            .expect("dst_folder to converted via to_str"),
    )?;
    variables.insert("ffizer_src_uri", ctx.cmd_opt.src.uri.raw.clone())?;
    variables.insert("ffizer_src_rev", ctx.cmd_opt.src.rev.clone())?;

    ctx.cmd_opt
        .key_value
        .iter()
        .map(|(k, v)| variables.insert(k, Variables::value_from_str(&v)?))
        .collect::<Result<Vec<()>>>()?;
    Ok(variables)
}

/// list actions to execute
fn plan(ctx: &Ctx, source_files: Vec<SourceFile>, variables: &Variables) -> Result<Vec<Action>> {
    // TODO create a map (dst_path, Vec<src_path>) src_path keep the order of application (from template layer)
    // TODO change Action into enum ?
    // TODO AddFile/UpdateFile can support a list of src_path
    let list_dst_and_src = source_files
        .into_iter()
        .map(|source_file| {
            compute_dst_path(ctx, &source_file.childpath(), variables)
                .map(|dst_path| (dst_path, source_file))
        })
        .collect::<Result<Vec<_>>>()?;
    // group by destination
    let srcs_by_dst = list_dst_and_src.into_iter().fold(
        std::collections::HashMap::<ChildPath, Vec<SourceFile>>::new(),
        |mut acc, l| {
            if let Some(x) = acc.get_mut(&l.0) {
                x.push(l.1);
            } else {
                acc.insert(l.0, vec![l.1]);
            }
            acc
        },
    );
    //actions.dedup_by(|a, b| PathBuf::from(&a.dst_path) == PathBuf::from(&b.dst_path));
    let mut actions = srcs_by_dst
        .into_iter()
        .map(|(dst_path, mut src)| {
            source_file::optimize_sourcefiles(&mut src);
            let operation = select_operation(ctx, &src, &dst_path);
            Action {
                //TODO reduce src (remove useless source) + test
                //TODO add SourceFile of existing file
                //TODO select the right operation
                dst_path,
                src,
                operation,
            }
        })
        .filter(|a| !a.src.is_empty())
        .collect::<Vec<_>>();
    // sort to have folder before files inside it (and mkdir berfore create file)
    actions.sort_by_key(|a| a.dst_path.relative.clone());
    Ok(actions)
}

//TODO accumulate Result (and error)
fn execute(ctx: &Ctx, actions: &[Action], variables: &Variables) -> Result<()> {
    use indicatif::ProgressBar;

    let pb = ProgressBar::new(actions.len() as u64);
    let mut handlebars = new_hbs();
    debug!(ctx.logger, "execute"; "variables" => ?&variables);

    for a in pb.wrap_iter(actions.iter()) {
        match a.operation {
            FileOperation::Nothing => (),
            FileOperation::Ignore => (),
            // TODO bench performance vs create_dir (and keep create_dir_all for root aka relative is empty)
            FileOperation::MkDir => {
                let path = PathBuf::from(&a.dst_path);
                fs::create_dir_all(&path).map_err(|source| Error::CreateFolder { path, source })?;
                copy_file_permissions(
                    PathBuf::from(a.src[0].childpath()),
                    PathBuf::from(&a.dst_path),
                )?
            }
            FileOperation::AddFile => {
                mk_file_on_action(&mut handlebars, variables, &a, "").map(|_| ())?
            }
            FileOperation::UpdateFile => {
                //TODO what to do if .LOCAL, .REMOTE already exist ?
                let (local, remote) = mk_file_on_action(&mut handlebars, variables, &a, ".REMOTE")?;
                let local_digest =
                    md5::compute(fs::read(&local).map_err(|source| Error::ReadFile {
                        path: local.clone(),
                        source,
                    })?);
                let remote_digest =
                    md5::compute(fs::read(&remote).map_err(|source| Error::ReadFile {
                        path: remote.clone(),
                        source,
                    })?);
                if local_digest == remote_digest {
                    fs::remove_file(&remote).map_err(|source| Error::RemoveFile {
                        path: remote.clone(),
                        source,
                    })?
                } else {
                    update_file(
                        //FIXME to use all the source
                        &PathBuf::from(a.src[0].childpath()),
                        &local,
                        &remote,
                        &ctx.cmd_opt.update_mode,
                    )?
                }
            }
        }
    }
    Ok(())
}

fn mk_file_on_action(
    handlebars: &mut handlebars::Handlebars,
    variables: &Variables,
    a: &Action,
    dest_suffix_ext: &str,
) -> Result<(PathBuf, PathBuf)> {
    let mut variables = variables.clone();
    let dest_full_path_target = PathBuf::from(&a.dst_path);
    let dest_full_path = files::add_suffix(&dest_full_path_target, dest_suffix_ext)?;
    let mut srcs = a.src.clone();
    srcs.reverse();
    let mut input_content: Vec<u8> = Vec::with_capacity(0);
    let index_latest = srcs.len() - 1;
    // based of the fact that list of source_files follow one of this configuration
    // - [RawFile]
    // - [RenderableFile+,RawFile{0,1}]
    for (i, source_file) in srcs.into_iter().enumerate() {
        let src_full_path = PathBuf::from(&source_file.childpath);
        match source_file.metadata {
            SourceFileMetadata::RawFile => {
                if i == index_latest {
                    fs::copy(&src_full_path, &dest_full_path).map_err(|source| {
                        Error::CopyFile {
                            src: src_full_path.clone(),
                            dst: dest_full_path.clone(),
                            source,
                        }
                    })?;
                } else {
                    input_content = fs::read(&src_full_path).map_err(|source| Error::ReadFile {
                        path: src_full_path.clone(),
                        source,
                    })?;
                }
            }
            SourceFileMetadata::RenderableFile { .. } => {
                if i == 0 && dest_full_path_target.exists() {
                    input_content =
                        fs::read(&dest_full_path_target).map_err(|source| Error::ReadFile {
                            path: dest_full_path_target.clone(),
                            source,
                        })?;
                }
                variables.insert("input_content", String::from_utf8_lossy(&input_content))?;
                render_template(handlebars, &variables, &src_full_path, &mut input_content)?;
                if i == index_latest {
                    fs::write(&dest_full_path, &input_content).map_err(|source| {
                        Error::WriteFile {
                            path: dest_full_path.clone(),
                            source,
                        }
                    })?;
                }
            }
            _ => (), // TODO return error,
        }
        if i == index_latest {
            copy_file_permissions(&src_full_path, &dest_full_path)?;
        }
    }
    Ok((PathBuf::from(&dest_full_path_target), dest_full_path))
}

fn render_template(
    handlebars: &mut handlebars::Handlebars,
    variables: &Variables,
    src_full_path: &PathBuf,
    output: &mut Vec<u8>,
) -> Result<()> {
    let src_name = &src_full_path.to_string_lossy();
    handlebars
        .register_template_file(&src_name, &src_full_path)
        .map_err(|e| match e {
            handlebars::TemplateFileError::TemplateError(err) => {
                handlebars::TemplateRenderError::from(err)
            }
            handlebars::TemplateFileError::IOError(err, msg) => {
                handlebars::TemplateRenderError::IOError(err, msg)
            }
        })
        .map_err(|source| Error::Handlebars {
            when: format!("load content of template '{:?}'", &src_full_path),
            template: src_name.to_string(),
            source,
        })?;
    output.clear(); //vec![u8] writer appends content if not clear
    handlebars
        .render_to_write(&src_name, &variables, output)
        .map_err(handlebars::TemplateRenderError::from)
        .map_err(|source| Error::Handlebars {
            when: "render template into buffer".into(),
            template: src_name.to_string(),
            source,
        })?;
    Ok(())
}

fn copy_file_permissions<P1, P2>(src: P1, dst: P2) -> Result<()>
where
    P1: AsRef<std::path::Path>,
    P2: AsRef<std::path::Path>,
{
    let src = src.as_ref();
    let dst = dst.as_ref();
    let perms = fs::metadata(&src)
        .map_err(|source| Error::CopyFilePermission {
            src: src.into(),
            dst: dst.into(),
            source,
        })?
        .permissions();
    fs::set_permissions(&dst, perms).map_err(|source| Error::CopyFilePermission {
        src: src.into(),
        dst: dst.into(),
        source,
    })?;
    Ok(())
}

fn update_file<P>(src: P, local: P, remote: P, mode_init: &UpdateMode) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    let mut mode = mode_init.to_owned();
    let remote = remote.as_ref();
    let local = local.as_ref();
    let src = src.as_ref();
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
                fs::remove_file(&local).map_err(|source| Error::RemoveFile {
                    path: local.into(),
                    source,
                })?;
                fs::rename(&remote, &local).map_err(|source| Error::RenameFile {
                    src: remote.into(),
                    dst: local.into(),
                    source,
                })?;
                break;
            }
            UpdateMode::Keep => {
                fs::remove_file(&remote).map_err(|source| Error::RemoveFile {
                    path: remote.into(),
                    source,
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
                fs::rename(&local, &new_local).map_err(|source| Error::RenameFile {
                    src: local.into(),
                    dst: new_local,
                    source,
                })?;
                fs::rename(&remote, &local).map_err(|source| Error::RenameFile {
                    src: remote.into(),
                    dst: local.into(),
                    source,
                })?;
                break;
            }
            UpdateMode::Merge => match merge_file(src, local, remote) {
                Ok(_) => {
                    fs::remove_file(&remote).map_err(|source| Error::RemoveFile {
                        path: remote.into(),
                        source,
                    })?;
                    break;
                }
                Err(_) => mode = UpdateMode::Ask,
            },
        }
    }
    Ok(())
}

fn merge_file<P>(src: P, local: P, remote: P) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    let remote = remote.as_ref();
    let local = local.as_ref();
    let src = src.as_ref();
    let merge_cmd = git::find_cmd_tool("merge").map_err(|source| Error::GitFindConfig {
        key: "merge".into(),
        source,
    })?;
    let new_local = files::add_suffix(&local, ".LOCAL")?;
    fs::copy(&local, &new_local).map_err(|source| Error::CopyFile {
        src: local.into(),
        dst: new_local.clone(),
        source,
    })?;
    let cmd_all = merge_cmd
        .replace("$REMOTE", &remote.to_string_lossy())
        .replace("$LOCAL", &new_local.to_string_lossy())
        .replace("$BASE", &src.to_string_lossy())
        .replace("$MERGED", &local.to_string_lossy());
    let cmd = cmd_all.split(' ').collect::<Vec<_>>();
    //dbg!(&cmd);
    std::process::Command::new(cmd[0])
        .args(&cmd[1..])
        // .stdin(std::process::Stdio::piped())
        // .stdout(std::process::Stdio::piped())
        .output()
        .map_err(|source| Error::RunCommand {
            cmd: cmd_all,
            source,
        })?;
    fs::remove_file(&new_local).map_err(|source| Error::RemoveFile {
        path: new_local,
        source,
    })?;
    Ok(())
}

//TODO optimize / bench to avoid re-creation of handlebars at each call
fn compute_dst_path(ctx: &Ctx, src: &ChildPath, variables: &Variables) -> Result<ChildPath> {
    let rendered_relative = src
        .relative
        .to_str()
        .ok_or(Error::Unknown("failed to stringify path".to_owned()))
        .and_then(|s| {
            let p = if !s.contains('{') {
                s.to_owned()
            } else {
                let handlebars = new_hbs();
                handlebars
                    .render_template(&s, variables)
                    .map_err(|source| Error::Handlebars {
                        when: format!("define path for '{:?}'", src),
                        template: s.into(),
                        source,
                    })?
            };
            Ok(PathBuf::from(p))
        })?;
    let relative = files::remove_special_suffix(&rendered_relative)?;

    Ok(ChildPath {
        base: ctx.cmd_opt.dst_folder.clone(),
        relative,
    })
}

fn select_operation(_ctx: &Ctx, sources: &[SourceFile], dst_path: &ChildPath) -> FileOperation {
    //FIXME to use all the sources
    let src_full_path = PathBuf::from(sources[0].childpath());
    let dest_full_path = PathBuf::from(dst_path);
    if dest_full_path.exists() {
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

fn run_scripts(ctx: &Ctx, template_composite: &TemplateComposite) -> Result<()> {
    do_in_folder(&ctx.cmd_opt.dst_folder, || {
        for (loc, scripts) in template_composite.find_scripts()? {
            for script in &scripts {
                if let Some(message) = &script.message {
                    ui::show_message(ctx, loc, message)?;
                }
                if let Some(cmd) = &script.cmd {
                    if ui::confirm_run_script(ctx, loc, cmd)? {
                        if let Err(err) = script.run() {
                            warn!(ctx.logger, ""; "err" => format!("{:#?}",err));
                        }
                    }
                }
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    pub use crate::cli_opt::*;
    use spectral::prelude::*;
    use tempfile::TempDir;

    const DST_FOLDER_STR: &str = "test/dst";
    const CONTENT_BASE: &str = "{{ base }}";
    const CONTENT_LOCAL: &str = "local";
    const CONTENT_REMOTE: &str = "remote";

    fn new_ctx_for_test() -> Ctx {
        Ctx {
            cmd_opt: ApplyOpts {
                dst_folder: PathBuf::from(DST_FOLDER_STR),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn new_variables_for_test() -> Variables {
        let mut variables = Variables::default();
        variables.insert("prj", "myprj").expect("insert prj");
        variables.insert("base", "remote").expect("insert base");
        variables
    }

    #[test]
    fn test_compute_dst_path_asis() {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();
        let src = ChildPath::new("test/src", "hello/sample.txt");
        let expected = ChildPath::new(DST_FOLDER_STR, "hello/sample.txt");
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn test_compute_dst_path_ffizer_handlebars() {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();
        let src = ChildPath::new("test/src", "hello/sample.txt.ffizer.hbs");
        let expected = ChildPath::new(DST_FOLDER_STR, "hello/sample.txt");
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn test_compute_dst_path_rendered_filename() {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();

        let src = ChildPath::new("test/src", "hello/{{ prj }}.txt");
        let expected = ChildPath::new(DST_FOLDER_STR, "hello/myprj.txt");
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_that!(&actual).is_equal_to(&expected);
    }

    #[test]
    fn test_compute_dst_path_rendered_folder() {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();

        let src = ChildPath::new("test/src", "hello/{{ prj }}/sample.txt");
        let expected = ChildPath::new(DST_FOLDER_STR, "hello/myprj/sample.txt");
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
    fn test_plan_with_empty() -> Result<(), Box<dyn std::error::Error>> {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();

        let sources: Vec<SourceFile> = vec![];
        let actions = plan(&ctx, sources, &variables)?;
        assert_that!(&actions).is_empty();
        Ok(())
    }

    #[test]
    fn test_plan_with_duplicate_from_2_templates() -> Result<(), Box<dyn std::error::Error>> {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();

        let sources: Vec<SourceFile> = vec![
            SourceFile::from((ChildPath::new("test/src1", "hello/file1.txt"), 1)),
            SourceFile::from((ChildPath::new("test/src2", "hello/file1.txt"), 2)),
        ];
        let actions = plan(&ctx, sources, &variables)?;
        let expected = vec![Action {
            src: vec![SourceFile::from((
                ChildPath::new("test/src1", "hello/file1.txt"),
                1,
            ))],
            dst_path: ChildPath::new(DST_FOLDER_STR, "hello/file1.txt"),
            operation: FileOperation::AddFile,
        }];
        assert_that!(&actions).is_equal_to(&expected);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_copy_file_permissions() {
        use std::os::unix::fs::PermissionsExt;
        // Create a directory inside of `std::env::temp_dir()`
        let tmp_dir = TempDir::new().expect("create a temp dir");
        let src_path = tmp_dir.path().join("src.txt");
        fs::write(&src_path, CONTENT_BASE).expect("create src file");
        let dst_path = tmp_dir.path().join("dst.txt");
        fs::write(&dst_path, CONTENT_BASE).expect("create dst file");

        let mut src_perms = fs::metadata(&src_path).unwrap().permissions();
        assert_that(&(src_perms.mode() & 0o100)).is_equal_to(0);
        src_perms.set_mode(src_perms.mode() | 0o100);
        assert_that(&(src_perms.mode() & 0o100)).is_equal_to(0o100);
        fs::set_permissions(&src_path, src_perms).expect("to set permissions");

        assert_that!(fs::metadata(&dst_path).unwrap().permissions())
            .is_not_equal_to(fs::metadata(&src_path).unwrap().permissions());

        let dst_perms = fs::metadata(&dst_path).unwrap().permissions();
        assert_that(&(dst_perms.mode() & 0o100)).is_equal_to(0);
        copy_file_permissions(&src_path, &dst_path).expect("copy file permissions");
        let dst_perms = fs::metadata(&dst_path).unwrap().permissions();
        assert_that(&(dst_perms.mode() & 0o100)).is_equal_to(0o100);

        assert_that!(fs::metadata(&dst_path).unwrap().permissions())
            .is_equal_to(fs::metadata(&src_path).unwrap().permissions());
    }

    #[test]
    fn test_mk_file_by_copy() {
        // Create a directory inside of `std::env::temp_dir()`
        let tmp_dir = TempDir::new().expect("create a temp dir");

        let src = ChildPath::new(tmp_dir.path(), "src.txt");
        let src_path = PathBuf::from(&src);
        fs::write(&src_path, CONTENT_BASE).expect("create src file");

        let dst = ChildPath::new(tmp_dir.path(), "dst.txt");
        let dst_path = PathBuf::from(&dst);

        let action = Action {
            dst_path: dst,
            src: vec![SourceFile::from((ChildPath::from(src), 0))],
            operation: FileOperation::AddFile,
        };

        let mut handlebars = new_hbs();
        let variables = new_variables_for_test();

        mk_file_on_action(&mut handlebars, &variables, &action, "").expect("mk_file is ok");
        assert_that!(&dst_path).exists();
        assert_that!(fs::read_to_string(&dst_path).unwrap()).is_equal_to(CONTENT_BASE.to_owned());
        assert_that!(fs::metadata(&dst_path).unwrap().permissions())
            .is_equal_to(fs::metadata(&src_path).unwrap().permissions());
    }

    #[test]
    fn test_mk_file_by_render() {
        // Create a directory inside of `std::env::temp_dir()`
        let tmp_dir = TempDir::new().expect("create a temp dir");

        let src = ChildPath::new(tmp_dir.path(), "src.txt.ffizer.hbs");
        let src_path = PathBuf::from(&src);
        fs::write(&src_path, CONTENT_BASE).expect("create src file");

        let dst = ChildPath::new(tmp_dir.path(), "dst.txt");
        let dst_path = PathBuf::from(&dst);

        let action = Action {
            dst_path: dst,
            src: vec![SourceFile::from((ChildPath::from(src), 0))],
            operation: FileOperation::AddFile,
        };

        let mut handlebars = new_hbs();
        let variables = new_variables_for_test();

        mk_file_on_action(&mut handlebars, &variables, &action, "").expect("mk_file is ok");
        assert_that!(&dst_path).exists();
        assert_that!(fs::read_to_string(&dst_path).unwrap()).is_equal_to(CONTENT_REMOTE.to_owned());
        assert_that!(fs::metadata(&dst_path).unwrap().permissions())
            .is_equal_to(fs::metadata(&src_path).unwrap().permissions());
    }

    fn setup_for_test_update() -> (TempDir, PathBuf, PathBuf, PathBuf) {
        // Create a directory inside of `std::env::temp_dir()`
        let tmp_dir = TempDir::new().expect("create a temp dir");

        let src_path = tmp_dir.path().join("src.txt.ffizer.hbs");
        fs::write(&src_path, CONTENT_BASE).expect("create src file");

        let local_path = tmp_dir.path().join("file.txt");
        fs::write(&local_path, CONTENT_LOCAL).expect("create local file");

        let remote_path = tmp_dir.path().join("file.txt.REMOTE");
        fs::write(&remote_path, CONTENT_REMOTE).expect("create remote file");

        (tmp_dir, local_path, remote_path, src_path)
    }

    #[test]
    fn test_update_file_override() {
        // grab _tmp_dir, because Drop will delete it and its files
        let (_tmp_dir, local_path, remote_path, src_path) = setup_for_test_update();
        update_file(&src_path, &local_path, &remote_path, &UpdateMode::Override)
            .expect("update without error");
        assert_that!(&local_path).exists();
        assert_that!(fs::read_to_string(&local_path).unwrap())
            .is_equal_to(CONTENT_REMOTE.to_owned());
        assert_that!(&remote_path).does_not_exist();
    }

    #[test]
    fn test_update_file_keep() {
        // grab _tmp_dir, because Drop will delete it and its files
        let (_tmp_dir, local_path, remote_path, src_path) = setup_for_test_update();
        update_file(&src_path, &local_path, &remote_path, &UpdateMode::Keep)
            .expect("update without error");
        assert_that!(&local_path).exists();
        assert_that!(fs::read_to_string(&local_path).unwrap())
            .is_equal_to(CONTENT_LOCAL.to_owned());
        assert_that!(&remote_path).does_not_exist();
    }

    #[test]
    fn test_update_file_update_as_remote() {
        // grab _tmp_dir, because Drop will delete it and its files
        let (_tmp_dir, local_path, remote_path, src_path) = setup_for_test_update();
        update_file(
            &src_path,
            &local_path,
            &remote_path,
            &UpdateMode::UpdateAsRemote,
        )
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
        let (_tmp_dir, local_path, remote_path, src_path) = setup_for_test_update();
        update_file(
            &src_path,
            &local_path,
            &remote_path,
            &UpdateMode::CurrentAsLocal,
        )
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
    //     let (_tmp_dir, local_path, remote_path, src_path) = setup_for_test_update();
    //     update_file(&src_path, &local_path, &remote_path, &UpdateMode::ShowDiff)
    //         .expect("update without error");
    // }
}
