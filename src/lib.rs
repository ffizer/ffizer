// #![feature(backtrace)]

#[macro_use]
extern crate serde;

pub mod error;
pub mod tools;

mod cfg;
mod cli_opt;
mod ctx;
mod files;
mod git;
mod graph;
mod path_pattern;
mod scripts;
mod source_file;
mod source_loc;
mod source_uri;
mod timeline;
mod ui;
mod variable_def;
mod variables;

pub use crate::cfg::provide_json_schema;
pub use crate::cli_opt::*;
pub use crate::path_pattern::PathPattern;
pub use crate::source_loc::SourceLoc;
pub use crate::source_uri::SourceUri;

use crate::cfg::{render_composite, TemplateComposite, VariableValueCfg};
use crate::error::*;
use crate::files::ChildPath;
use crate::source_file::{SourceFile, SourceFileMetadata};
use crate::variables::Variables;
use handlebars_misc_helpers::new_hbs;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use timeline::FileMeta;
use tracing::{debug, warn};

pub(crate) const IGNORED_FOLDER_PREFIX: &str = "_ffizer_ignore";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOperation {
    Nothing,
    Ignore,
    MkDir,
    AddFile,
    UpdateFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub src: Vec<SourceFile>,
    pub dst_path: ChildPath,
    // template: TemplateDef,
    pub operation: FileOperation,
}

#[derive(Debug, Clone, Default)]
pub struct Ctx {
    pub cmd_opt: ApplyOpts,
}

pub fn reprocess(cmd_opt: ReapplyOpts) -> Result<()> {
    let temp_dir = TempDir::with_prefix(IGNORED_FOLDER_PREFIX)?;

    let tmp_template = timeline::make_template_from_folder(&cmd_opt.dst_folder, temp_dir.path())?;
    let new_ctx = Ctx {
        cmd_opt: ApplyOpts {
            confirm: cmd_opt.confirm,
            src: tmp_template,
            update_mode: cmd_opt.update_mode,
            no_interaction: cmd_opt.no_interaction,
            dst_folder: cmd_opt.dst_folder,
            offline: cmd_opt.offline,
            key_value: cmd_opt.key_value,
        },
    };
    process(&new_ctx)?;
    Ok(temp_dir.close()?)
}

pub fn process(ctx: &Ctx) -> Result<()> {
    debug!("extracting variables from context",);
    let mut variables = ctx::extract_variables(ctx)?;
    debug!("compositing templates");

    let mut template_composite =
        TemplateComposite::from_src(&variables.src, ctx.cmd_opt.offline, &ctx.cmd_opt.src)?;

    let mut confirmed_variables = variables.cli;
    confirmed_variables.append(&mut variables.src);
    let confirmed_variables = confirmed_variables; // make immutable

    debug!(confirmed_variables = ?confirmed_variables, "asking variables");
    let mut variable_configs = template_composite.find_variablecfgs()?;

    // Updates defaults with suggested variables before asking.
    variable_configs.iter_mut().for_each(|cfg| {
        if let Some(v) = variables.saved.get(&cfg.name) {
            cfg.default_value = Some(VariableValueCfg(v.clone()))
        }
    });
    let variable_configs = variable_configs; // make immutable

    let used_variables = ui::ask_variables(ctx, &variable_configs, confirmed_variables)?;
    // update cfg(s) with variables defined by user (use to update ignore, scripts,...)
    debug!(variables = ?used_variables, "update template_composite with variables");
    template_composite = render_composite(&template_composite, &used_variables, true)?;
    debug!("listing files from templates");
    let source_files = template_composite.find_sourcefiles()?;
    debug!("defining plan of rendering");
    let actions = plan(ctx, source_files, &used_variables)?;

    if ui::confirm_plan(ctx, &actions)? {
        debug!("executing plan of rendering");
        let new_metas = execute(ctx, &actions, &used_variables)?;
        debug!("running scripts");
        run_scripts(ctx, &template_composite)?;
        debug!("Saving metadata");
        timeline::save_options(&used_variables, &ctx.cmd_opt.src, &ctx.cmd_opt.dst_folder)?;

        timeline::save_metas_for_source(new_metas, &ctx.cmd_opt.dst_folder, "global".into())?;
    }
    Ok(())
}

fn do_in_folder<F, R>(folder: &Path, f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    fs::create_dir_all(folder).map_err(|source| Error::CreateFolder {
        path: folder.to_path_buf(),
        source,
    })?;
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(folder)?;
    // let res = apply_plan(&ctx, &actions, &variables, &template_composite);
    let res = f();
    std::env::set_current_dir(current_dir)?;
    res
}

/// list actions to execute
fn plan(ctx: &Ctx, source_files: Vec<SourceFile>, variables: &Variables) -> Result<Vec<Action>> {
    // TODO create a map (dst_path, Vec<src_path>) src_path keep the order of application (from template layer)
    // TODO change Action into enum ?
    // TODO AddFile/UpdateFile can support a list of src_path
    let list_dst_and_src = source_files
        .into_iter()
        .map(|source_file| {
            compute_dst_path(ctx, source_file.childpath(), variables)
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
                src,
                dst_path,
                operation,
            }
        })
        .filter(|a| !a.src.is_empty())
        .collect::<Vec<_>>();
    // sort to have folder before files inside it (and mkdir berfore create file)
    actions.sort_by_key(|a| a.dst_path.relative.clone());
    Ok(actions)
}

fn decide_update_mode(
    local: &FileMeta,
    remote: &FileMeta,
    past_remote: &FileMeta,
    past_final: &FileMeta,
) -> UpdateMode {
    if past_remote == remote {
        // keep if the template hasn't changed since last time
        UpdateMode::Keep
    } else if past_final == past_remote && past_final == local {
        // override if the user accepted remote last time and hasn't changed anything since
        UpdateMode::Override
    } else {
        UpdateMode::Ask
    }
}

//TODO accumulate Result (and error)
fn execute(
    ctx: &Ctx,
    actions: &[Action],
    variables: &Variables,
) -> Result<Vec<(FileMeta, FileMeta)>> {
    use indicatif::ProgressBar;

    let pb = ProgressBar::new(actions.len() as u64);
    let mut handlebars = new_hbs();
    debug!(?variables, "execute");

    // let source = &ctx.cmd_opt.src;
    let target_folder = &ctx.cmd_opt.dst_folder;
    let past_metas = timeline::get_stored_metas_for_source(target_folder, "global".into())?;
    let mut new_metas = Vec::new();

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
                let (_, remote) = mk_file_on_action(&mut handlebars, variables, a, "")?;
                let remote_meta =
                    timeline::get_meta(target_folder, remote.strip_prefix(target_folder)?)?;
                new_metas.push((remote_meta.clone(), remote_meta));
            }
            FileOperation::UpdateFile => {
                //TODO what to do if .LOCAL, .REMOTE already exist ?
                let (local_path, remote_path) =
                    mk_file_on_action(&mut handlebars, variables, a, ".REMOTE")?;

                let local =
                    timeline::get_meta(target_folder, local_path.strip_prefix(target_folder)?)?;
                let key = &local.key;

                let remote =
                    timeline::get_meta(target_folder, remote_path.strip_prefix(target_folder)?)?
                        .with_key(key);

                // If there are no changes, skip update
                if local == remote {
                    fs::remove_file(&remote_path).map_err(|source| Error::RemoveFile {
                        path: remote_path.clone(),
                        source,
                    })?;
                    new_metas.push((remote.clone(), remote));
                    continue;
                }

                let update_mode = match past_metas.get(key) {
                    Some((past_remote, past_final))
                        if ctx.cmd_opt.update_mode == UpdateMode::Auto =>
                    {
                        decide_update_mode(&local, &remote, past_remote, past_final)
                    }
                    _ => ctx.cmd_opt.update_mode.clone(),
                };

                update_file(
                    //FIXME to use all the source
                    &PathBuf::from(a.src[0].childpath()),
                    &local_path,
                    &remote_path,
                    &update_mode,
                )?;

                new_metas.push((remote, timeline::get_meta(
                    target_folder,
                    local_path.strip_prefix(target_folder)?,
                )?));
            }
        }
    }
    Ok(new_metas)
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
    src_full_path: &Path,
    output: &mut Vec<u8>,
) -> Result<()> {
    let src_name = &src_full_path.to_string_lossy();
    handlebars
        .register_template_file(src_name, src_full_path)
        .map_err(handlebars::RenderError::from)
        .map_err(|source| Error::Handlebars {
            when: format!("load content of template '{:?}'", &src_full_path),
            template: Box::new(src_name.to_string()),
            source: Box::new(source),
        })?;
    output.clear(); //vec![u8] writer appends content if not clear
    handlebars
        .render_to_write(src_name, &variables, output)
        .map_err(handlebars::RenderError::from)
        .map_err(|source| Error::Handlebars {
            when: "render template into buffer".into(),
            template: Box::new(src_name.to_string()),
            source: Box::new(source),
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
    let perms = fs::metadata(src)
        .map_err(|source| Error::CopyFilePermission {
            src: src.into(),
            dst: dst.into(),
            source,
        })?
        .permissions();
    fs::set_permissions(dst, perms).map_err(|source| Error::CopyFilePermission {
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
            UpdateMode::Auto => {
                // should not enter here
                mode = UpdateMode::Ask;
            }
            UpdateMode::Ask => {
                mode = ui::ask_update_mode(local)?;
            }
            UpdateMode::ShowDiff => {
                // show diff (then re-ask)
                ui::show_difference(&local, &remote)?;
                mode = UpdateMode::Ask;
            }
            UpdateMode::Override => {
                fs::remove_file(local).map_err(|source| Error::RemoveFile {
                    path: local.into(),
                    source,
                })?;
                fs::rename(remote, local).map_err(|source| Error::RenameFile {
                    src: remote.into(),
                    dst: local.into(),
                    source,
                })?;
                break;
            }
            UpdateMode::Keep => {
                fs::remove_file(remote).map_err(|source| Error::RemoveFile {
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
                let new_local = files::add_suffix(local, ".LOCAL")?;
                fs::rename(local, &new_local).map_err(|source| Error::RenameFile {
                    src: local.into(),
                    dst: new_local,
                    source,
                })?;
                fs::rename(remote, local).map_err(|source| Error::RenameFile {
                    src: remote.into(),
                    dst: local.into(),
                    source,
                })?;
                break;
            }
            UpdateMode::Merge => match merge_file(src, local, remote) {
                Ok(_) => {
                    fs::remove_file(remote).map_err(|source| Error::RemoveFile {
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
    let new_local = files::add_suffix(local, ".LOCAL")?;
    fs::copy(local, &new_local).map_err(|source| Error::CopyFile {
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
        .ok_or_else(|| Error::Unknown("failed to stringify path".to_owned()))
        .and_then(|s| {
            let p = if !s.contains('{') {
                s.to_owned()
            } else {
                let handlebars = new_hbs();
                handlebars
                    .render_template(s, variables)
                    .map_err(|source| Error::Handlebars {
                        when: format!("define path for '{:?}'", src),
                        template: Box::new(s.into()),
                        source: Box::new(source),
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
                    if ui::confirm_run_script(ctx, loc, cmd, script.default_confirm_answer)? {
                        if let Err(err) = script.run() {
                            warn!(?err);
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
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    const DST_FOLDER_STR: &str = "test/dst";
    const CONTENT_BASE: &str = "{{ base }}";
    const CONTENT_LOCAL: &str = "local";
    const CONTENT_REMOTE: &str = "remote";

    fn new_variables_for_test() -> Variables {
        let mut variables = Variables::default();
        variables.insert("prj", "myprj").expect("insert prj");
        variables.insert("base", "remote").expect("insert base");
        variables
    }

    pub fn new_ctx_from<T: Into<PathBuf>>(dst: T) -> Ctx {
        Ctx {
            cmd_opt: ApplyOpts {
                dst_folder: dst.into(),
                ..Default::default()
            },
        }
    }

    fn new_ctx_for_test() -> Ctx {
        new_ctx_from(DST_FOLDER_STR)
    }

    #[test]
    fn test_compute_dst_path_asis() {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();
        let src = ChildPath::new("test/src", "hello/sample.txt");
        let expected = ChildPath::new(DST_FOLDER_STR, "hello/sample.txt");
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_eq!(&expected, &actual);
    }

    #[test]
    fn test_compute_dst_path_ffizer_handlebars() {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();
        let src = ChildPath::new("test/src", "hello/sample.txt.ffizer.hbs");
        let expected = ChildPath::new(DST_FOLDER_STR, "hello/sample.txt");
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_eq!(&expected, &actual);
    }

    #[test]
    fn test_compute_dst_path_rendered_filename() {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();

        let src = ChildPath::new("test/src", "hello/{{ prj }}.txt");
        let expected = ChildPath::new(DST_FOLDER_STR, "hello/myprj.txt");
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_eq!(&expected, &actual);
    }

    #[test]
    fn test_compute_dst_path_rendered_folder() {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();

        let src = ChildPath::new("test/src", "hello/{{ prj }}/sample.txt");
        let expected = ChildPath::new(DST_FOLDER_STR, "hello/myprj/sample.txt");
        let actual = compute_dst_path(&ctx, &src, &variables).unwrap();
        assert_eq!(&expected, &actual);
    }

    #[test]
    fn test_path_extension_extraction() {
        use std::ffi::OsStr;

        assert_eq!(
            Some(OsStr::new("ext1")),
            PathBuf::from("foo.ext1").extension()
        );
        assert_eq!(
            Some(OsStr::new("ext1")),
            PathBuf::from("foo.ext2.ext1").extension()
        );
    }

    #[test]
    fn test_plan_with_empty() -> Result<(), Box<dyn std::error::Error>> {
        let ctx = new_ctx_for_test();
        let variables = new_variables_for_test();

        let sources: Vec<SourceFile> = vec![];
        let actions = plan(&ctx, sources, &variables)?;
        assert_eq!(true, actions.is_empty());
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
        assert_eq!(&expected, &actions);
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
        assert_eq!(0, (src_perms.mode() & 0o100));
        src_perms.set_mode(src_perms.mode() | 0o100);
        assert_eq!(0o100, (src_perms.mode() & 0o100));
        fs::set_permissions(&src_path, src_perms).expect("to set permissions");

        assert_ne!(
            fs::metadata(&src_path).unwrap().permissions(),
            fs::metadata(&dst_path).unwrap().permissions()
        );

        let dst_perms = fs::metadata(&dst_path).unwrap().permissions();
        assert_eq!(0, (dst_perms.mode() & 0o100));
        copy_file_permissions(&src_path, &dst_path).expect("copy file permissions");
        let dst_perms = fs::metadata(&dst_path).unwrap().permissions();
        assert_eq!(0o100, (dst_perms.mode() & 0o100));

        assert_eq!(
            fs::metadata(&src_path).unwrap().permissions(),
            fs::metadata(&dst_path).unwrap().permissions()
        );
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
            src: vec![SourceFile::from((src, 0))],
            operation: FileOperation::AddFile,
        };

        let mut handlebars = new_hbs();
        let variables = new_variables_for_test();

        mk_file_on_action(&mut handlebars, &variables, &action, "").expect("mk_file is ok");
        assert_eq!(true, dst_path.exists());
        assert_eq!(
            CONTENT_BASE.to_owned(),
            fs::read_to_string(&dst_path).unwrap()
        );
        assert_eq!(
            fs::metadata(&src_path).unwrap().permissions(),
            fs::metadata(&dst_path).unwrap().permissions()
        );
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
            src: vec![SourceFile::from((src, 0))],
            operation: FileOperation::AddFile,
        };

        let mut handlebars = new_hbs();
        let variables = new_variables_for_test();

        mk_file_on_action(&mut handlebars, &variables, &action, "").expect("mk_file is ok");
        assert_eq!(true, dst_path.exists());
        assert_eq!(
            CONTENT_REMOTE.to_owned(),
            fs::read_to_string(&dst_path).unwrap()
        );
        assert_eq!(
            fs::metadata(&src_path).unwrap().permissions(),
            fs::metadata(&dst_path).unwrap().permissions()
        );
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
        assert_eq!(true, local_path.exists());
        assert_eq!(
            CONTENT_REMOTE.to_owned(),
            fs::read_to_string(&local_path).unwrap()
        );
        assert_eq!(false, remote_path.exists());
    }

    #[test]
    fn test_update_file_keep() {
        // grab _tmp_dir, because Drop will delete it and its files
        let (_tmp_dir, local_path, remote_path, src_path) = setup_for_test_update();
        update_file(&src_path, &local_path, &remote_path, &UpdateMode::Keep)
            .expect("update without error");
        assert_eq!(true, local_path.exists());
        assert_eq!(
            CONTENT_LOCAL.to_owned(),
            fs::read_to_string(&local_path).unwrap()
        );
        assert_eq!(false, remote_path.exists());
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
        assert_eq!(true, local_path.exists());
        assert_eq!(
            CONTENT_LOCAL.to_owned(),
            fs::read_to_string(&local_path).unwrap()
        );
        assert_eq!(true, remote_path.exists());
        assert_eq!(
            CONTENT_REMOTE.to_owned(),
            fs::read_to_string(&remote_path).unwrap()
        );
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
        assert_eq!(true, local_path.exists());
        assert_eq!(
            CONTENT_REMOTE.to_owned(),
            fs::read_to_string(&local_path).unwrap()
        );
        assert_eq!(false, remote_path.exists());
        let dot_local_path = files::add_suffix(&local_path, ".LOCAL").unwrap();
        assert_eq!(true, dot_local_path.exists());
        assert_eq!(
            CONTENT_LOCAL.to_owned(),
            fs::read_to_string(&dot_local_path).unwrap()
        );
    }

    // #[test]
    // fn test_update_file_show_diff() {
    //     // grab _tmp_dir, because Drop will delete it and its files
    //     let (_tmp_dir, local_path, remote_path, src_path) = setup_for_test_update();
    //     update_file(&src_path, &local_path, &remote_path, &UpdateMode::ShowDiff)
    //         .expect("update without error");
    // }

    #[test]
    fn test_hbs_expression() {
        let handlebars = new_hbs();
        let variables = new_variables_for_test();
        assert_eq!(
            handlebars.render_template("myprj", &variables).unwrap(),
            "myprj"
        );
        assert_eq!(
            handlebars.render_template("{{prj}}", &variables).unwrap(),
            "myprj"
        );
        // assert_eq!(
        //     handlebars
        //         .render_template("{{\"myprj\"}}", &variables)
        //         .unwrap(),
        //     "myprj"
        // );
        // assert_eq!(
        //     handlebars
        //         .render_template("{{'myprj'}}", &variables)
        //         .unwrap(),
        //     "myprj"
        // );
        assert_eq!(
            handlebars
                .render_template("{{#if prj}}v1{{/if}}", &variables)
                .unwrap(),
            "v1"
        );
        assert_eq!(
            handlebars
                .render_template("{{#unless prj}}v1{{/unless}}", &variables)
                .unwrap(),
            ""
        );
        assert_eq!(
            handlebars
                .render_template("{{#if (eq prj \"myprj\")}}v1{{/if}}{{prj}}", &variables)
                .unwrap(),
            "v1myprj"
        );
        assert_eq!(
            handlebars
                .render_template(
                    "{{#unless (eq prj \"myprj\")}}v1{{/unless}}{{prj}}",
                    &variables
                )
                .unwrap(),
            "myprj"
        );
        assert_eq!(
            handlebars
                .render_template("{{#if (eq prj \"foo\")}}v1{{/if}}{{prj}}", &variables)
                .unwrap(),
            "myprj"
        );
        assert_eq!(
            handlebars
                .render_template("{{#if (ne prj \"foo\")}}v1{{/if}}{{prj}}", &variables)
                .unwrap(),
            "v1myprj"
        );
        assert_eq!(
            handlebars
                .render_template("{{#if (not (eq prj \"foo\"))}}v1{{/if}}{{prj}}", &variables)
                .unwrap(),
            "v1myprj"
        );
        assert_eq!(
            handlebars
                .render_template("{{eq prj \"foo\"}}", &variables)
                .unwrap(),
            "false"
        );
        assert_eq!(
            handlebars
                .render_template("{{ne prj \"foo\"}}", &variables)
                .unwrap(),
            "true"
        );
        assert_eq!(
            handlebars
                .render_template("{{not (eq prj \"foo\")}}", &variables)
                .unwrap(),
            "true"
        );
        assert_eq!(
            handlebars
                .render_template("{{eq prj \"myprj\"}}", &variables)
                .unwrap(),
            "true"
        );
        assert_eq!(
            handlebars
                .render_template("{{ne prj \"myprj\"}}", &variables)
                .unwrap(),
            "false"
        );

        //WARNING: use double quote to enclose string, single quote are not converter as string
        // since handlebars 4.0 single quote generates "syntax error"
        // assert_eq!(
        //     handlebars
        //         .render_template("{{eq prj 'myprj'}}", &variables)
        //         .unwrap(),
        //     "false"
        // );

        //WARNING: no error raised if undefined variable is used into an expression
        // since handlebars 4.1 an error is raised
        // handlebars.set_strict_mode(true);
        // assert_eq!(
        //     handlebars
        //         .render_template("{{eq undefined \"myprj\"}}", &variables)
        //         .unwrap(),
        //     "false"
        // );
        // assert_eq!(
        //     handlebars
        //         .render_template("{{ne undefined \"myprj\"}}", &variables)
        //         .unwrap(),
        //     "true"
        // );
    }
}
