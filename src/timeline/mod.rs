use self::persist::*;
use crate::cfg::{ImportCfg, TemplateCfg};
use crate::variables::Variables;
use crate::{Result, SourceLoc, SourceUri};
use std::fs;
use std::path::Path;
use std::str::FromStr;

mod persist;

pub(crate) const FFIZER_DATASTORE_DIRNAME: &str = ".ffizer";
const OPTIONS_FILENAME: &str = "options.yaml";
const VERSION_FILENAME: &str = "version.txt";

pub(crate) fn make_template(options: PersistedOptions) -> TemplateCfg {
    // not ready for standalone command, only used as part of reapply for now
    // does not use variables for now, it relies on the variables reimport during apply
    let imports: Vec<ImportCfg> = options
        .sources
        .into_iter()
        .map(|source| ImportCfg {
            uri: source.uri,
            rev: source.rev,
            subfolder: source.subfolder.map(|x| x.to_string_lossy().to_string()),
        })
        .collect();

    TemplateCfg {
        imports,
        use_template_dir: true,
        ..Default::default() // No need to set default variables as they will be read from the target folder, this needs to be improved to make a true template
    }
}

pub(crate) fn make_template_from_folder(
    source_folder: &Path,
    target_folder: &Path,
) -> Result<SourceLoc> {
    let options = load_options(source_folder)?;
    let template_cfg = make_template(options);

    serde_yaml::to_writer(
        std::fs::File::create(target_folder.join(".ffizer.yaml"))?,
        &template_cfg,
    )?;
    std::fs::create_dir(target_folder.join("template"))?;

    Ok(SourceLoc {
        uri: SourceUri::from_str(&target_folder.to_string_lossy())?,
        rev: None,
        subfolder: None,
    })
}

fn key_from_loc(source: &SourceLoc) -> (String, String, String) {
    let uri = &source.uri;
    (
        uri.host.clone().unwrap_or_default(),
        uri.path.to_string_lossy().into(),
        source
            .subfolder
            .clone()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
    )
}

pub(crate) fn load_options(folder: &Path) -> Result<PersistedOptions> {
    let metadata_path = folder.join(FFIZER_DATASTORE_DIRNAME).join(OPTIONS_FILENAME);
    if metadata_path.exists() {
        Ok(serde_yaml::from_reader(std::fs::File::open(
            metadata_path,
        )?)?)
    } else {
        Ok(PersistedOptions::default())
    }
}

pub(crate) fn make_new_options(
    previous_opts: PersistedOptions,
    variables: &Variables,
    source: &SourceLoc,
) -> Result<PersistedOptions> {
    let variables_to_save: Vec<PersistedVariable> = {
        let mut vars: Variables = previous_opts.variables.try_into()?;
        vars.append(&mut variables.clone());
        vars.retain(|k, _v| !k.starts_with("ffizer_"));
        vars.into()
    };

    let previous_srcs: Vec<SourceLoc> = previous_opts
        .sources
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<SourceLoc>>>()?;

    let is_dummy = source.uri.path.file_name().is_some_and(|x| {
        x.to_string_lossy()
            .starts_with(crate::IGNORED_FOLDER_PREFIX)
    });

    let saved_srcs: Vec<PersistedSrc> = if is_dummy {
        previous_srcs.into_iter().map(Into::into).collect()
    } else {
        let new_key = key_from_loc(source);

        vec![source.clone()]
            .into_iter()
            .chain(
                previous_srcs
                    .into_iter()
                    .filter(|loc| key_from_loc(loc) != new_key),
            )
            .map(Into::into)
            .collect()
    };
    Ok(PersistedOptions {
        variables: variables_to_save,
        sources: saved_srcs,
    })
}

pub(crate) fn save_options(
    variables: &Variables,
    source: &SourceLoc,
    dst_folder: &Path,
) -> Result<()> {
    let previous_options = load_options(dst_folder)?;

    let options = make_new_options(previous_options, variables, source)?;

    let ffizer_folder = dst_folder.join(FFIZER_DATASTORE_DIRNAME);
    if !ffizer_folder.exists() {
        std::fs::create_dir(&ffizer_folder)?;
    }
    // Save ffizer version
    fs::write(
        ffizer_folder.join(VERSION_FILENAME),
        env!("CARGO_PKG_VERSION"),
    )?;

    serde_yaml::to_writer(
        std::fs::File::create(ffizer_folder.join(OPTIONS_FILENAME))?,
        &options,
    )?;
    Ok(())
}

#[allow(dead_code)] // Used in testing
pub(crate) fn get_saved_sources(folder: &Path) -> Result<Vec<SourceLoc>> {
    load_options(folder)?
        .sources
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}

pub(crate) fn get_saved_variables(folder: &Path) -> Result<Variables> {
    load_options(folder)?.variables.try_into()
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use similar_asserts::assert_eq;
    use std::path::PathBuf;

    use super::persist;
    use super::*;
    use crate::Ctx;
    use crate::cli_opt::ApplyOpts;
    use crate::tests::new_ctx_from;
    use tempfile::TempDir;

    #[fixture]
    fn variables() -> Variables {
        let mut variables = Variables::default();
        variables.insert("prj", "myprj").expect("insert prj");
        variables.insert("base", "remote").expect("insert base");
        variables
    }

    #[fixture]
    fn tmp_dir() -> TempDir {
        TempDir::new().expect("create a temp dir")
    }

    fn new_ctx_from_src_dst(src: &SourceLoc, dst: &Path) -> Ctx {
        Ctx {
            cmd_opt: ApplyOpts {
                src: src.clone(),
                dst_folder: dst.to_path_buf(),
                ..Default::default()
            },
        }
    }

    #[rstest]
    fn test_save_load_variables(tmp_dir: TempDir, variables: Variables) {
        let ctx = new_ctx_from(tmp_dir.path());

        let mut variables_with_ffizer = variables.clone();
        variables_with_ffizer
            .insert("ffizer_version", "0.0.0")
            .unwrap();

        save_options(
            &variables_with_ffizer,
            &ctx.cmd_opt.src,
            &ctx.cmd_opt.dst_folder,
        )
        .unwrap();
        let saved_variables = get_saved_variables(&ctx.cmd_opt.dst_folder).unwrap();
        assert_eq!(saved_variables, variables);
    }

    mod test_save_load_srcs {
        use super::*;
        use similar_asserts::assert_eq;

        #[fixture]
        fn local_source() -> SourceLoc {
            SourceLoc {
                uri: SourceUri::from_str("local_path").unwrap(),
                rev: None,
                subfolder: Some(PathBuf::from_str("some_subfolder").unwrap()),
            }
        }

        #[fixture]
        fn remote_source() -> SourceLoc {
            SourceLoc {
                uri: SourceUri::from_str("http://blabla.truc/a/path").unwrap(),
                rev: None,
                subfolder: None,
            }
        }

        #[rstest]
        fn single_use(
            tmp_dir: TempDir,
            variables: Variables,
            #[from(local_source)] source_1: SourceLoc,
        ) {
            let ctx_1 = new_ctx_from_src_dst(&source_1, tmp_dir.path());

            save_options(&variables, &ctx_1.cmd_opt.src, &ctx_1.cmd_opt.dst_folder).unwrap();

            let saved_sources = get_saved_sources(&ctx_1.cmd_opt.dst_folder).unwrap();

            let expected = vec![source_1];
            assert_eq!(expected, saved_sources);
        }

        #[rstest]
        fn multi_use(
            tmp_dir: TempDir,
            variables: Variables,
            #[from(local_source)] source_1: SourceLoc,
            #[from(remote_source)] source_2: SourceLoc,
        ) {
            let ctx_1 = new_ctx_from_src_dst(&source_1, tmp_dir.path());
            let ctx_2 = new_ctx_from_src_dst(&source_2, tmp_dir.path());

            save_options(&variables, &ctx_1.cmd_opt.src, &ctx_1.cmd_opt.dst_folder).unwrap();
            save_options(&variables, &ctx_2.cmd_opt.src, &ctx_2.cmd_opt.dst_folder).unwrap();

            let saved_sources = get_saved_sources(&ctx_1.cmd_opt.dst_folder).unwrap();

            let expected = vec![source_2, source_1];
            assert_eq!(expected, saved_sources);
        }

        #[rstest]
        fn multi_use_with_replacement(
            tmp_dir: TempDir,
            variables: Variables,
            #[from(local_source)] source_1: SourceLoc,
        ) {
            let source_2 = SourceLoc {
                rev: Some("Some-other-branch".to_string()),
                ..source_1.clone()
            };

            let ctx_1 = new_ctx_from_src_dst(&source_1, tmp_dir.path());
            let ctx_2 = new_ctx_from_src_dst(&source_2, tmp_dir.path());

            save_options(&variables, &ctx_1.cmd_opt.src, &ctx_1.cmd_opt.dst_folder).unwrap();
            save_options(&variables, &ctx_2.cmd_opt.src, &ctx_2.cmd_opt.dst_folder).unwrap();

            let saved_sources = get_saved_sources(&ctx_1.cmd_opt.dst_folder).unwrap();

            let expected = vec![source_2];
            assert_eq!(expected, saved_sources);
        }
    }

    mod test_make_template {
        use super::*;
        use similar_asserts::assert_eq;

        #[rstest]
        fn empty() {
            let options = persist::PersistedOptions {
                variables: vec![],
                sources: vec![],
            };

            let expected = TemplateCfg {
                use_template_dir: true,
                ..Default::default()
            };

            assert_eq!(expected, make_template(options))
        }

        #[rstest]
        fn single_source() {
            let options = persist::PersistedOptions {
                variables: vec![],
                sources: vec![PersistedSrc {
                    uri: "path/to/foo".to_string(),
                    rev: None,
                    subfolder: None,
                }],
            };

            let expected = TemplateCfg {
                imports: vec![ImportCfg {
                    uri: "path/to/foo".to_string(),
                    rev: None,
                    subfolder: None,
                }],
                use_template_dir: true,
                ..Default::default()
            };
            assert_eq!(expected, make_template(options))
        }

        #[rstest]
        fn multi_source() {
            let options = persist::PersistedOptions {
                variables: vec![],
                sources: vec![
                    PersistedSrc {
                        uri: "path/to/foo".to_string(),
                        rev: None,
                        subfolder: None,
                    },
                    PersistedSrc {
                        uri: "http://blabla.truc/a/path".into(),
                        rev: Some("master".into()),
                        subfolder: Some(PathBuf::from_str("some_subfolder").unwrap()),
                    },
                ],
            };

            let expected = TemplateCfg {
                imports: vec![
                    ImportCfg {
                        uri: "path/to/foo".to_string(),
                        rev: None,
                        subfolder: None,
                    },
                    ImportCfg {
                        uri: "http://blabla.truc/a/path".into(),
                        rev: Some("master".into()),
                        subfolder: Some("some_subfolder".into()),
                    },
                ],
                use_template_dir: true,
                ..Default::default()
            };
            assert_eq!(expected, make_template(options))
        }
    }
}
