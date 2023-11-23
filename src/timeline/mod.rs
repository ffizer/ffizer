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

pub(crate) fn make_template(source_folder: &Path, target_folder: &Path) -> Result<SourceLoc> {
    // not ready for standalone command, only used as part of reapply for now
    let metadata_path = source_folder
        .join(FFIZER_DATASTORE_DIRNAME)
        .join(OPTIONS_FILENAME);
    let persisted: PersistedOptions =
        { serde_yaml::from_reader(std::fs::File::open(metadata_path)?)? };

    let imports: Vec<ImportCfg> = persisted
        .sources
        .into_iter()
        .map(|source| ImportCfg {
            uri: source.uri,
            rev: source.rev,
            subfolder: source.subfolder.map(|x| x.to_string_lossy().to_string()),
        })
        .collect();

    let template_cfg = TemplateCfg {
        imports,
        use_template_dir: true,
        ..Default::default() // No need to set default variables as they will be read from the target folder
    };
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

pub(crate) fn save_options(
    variables: &Variables,
    source: &SourceLoc,
    dst_folder: &Path,
) -> Result<()> {
    // Save or update default variable values stored in datastore
    let mut variables_to_save = get_saved_variables(dst_folder)?;
    variables_to_save.append(&mut variables.clone()); // update already existing keys
    variables_to_save.retain(|k, _v| !k.starts_with("ffizer_"));
    let mut saved_srcs: Vec<PersistedSrc>;
    if !source
        .uri
        .path
        .file_name()
        .is_some_and(|x| x.to_string_lossy().starts_with("dummy_"))
    {
        let new_key = key_from_loc(source);

        saved_srcs = vec![source.clone().into()];
        saved_srcs.extend(
            get_saved_sources(dst_folder)?
                .into_iter()
                .filter(|loc| key_from_loc(loc) != new_key)
                .map(|loc| loc.into()),
        );
    } else {
        saved_srcs = get_saved_sources(dst_folder)?
            .into_iter()
            .map(|loc| loc.into())
            .collect();
    }

    let persisted_options = PersistedOptions {
        variables: variables_to_save.into(),
        sources: saved_srcs,
    };

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
        &persisted_options,
    )?;
    Ok(())
}

pub(crate) fn get_saved_sources(folder: &Path) -> Result<Vec<SourceLoc>> {
    let metadata_path = folder.join(FFIZER_DATASTORE_DIRNAME).join(OPTIONS_FILENAME);
    let sources = if metadata_path.exists() {
        let persisted: PersistedOptions =
            { serde_yaml::from_reader(std::fs::File::open(metadata_path)?)? };

        persisted
            .sources
            .into_iter()
            .map(|v| -> Result<SourceLoc> { v.try_into() })
            .collect::<Result<Vec<SourceLoc>>>()?
    } else {
        Vec::default()
    };
    Ok(sources)
}

pub(crate) fn get_saved_variables(folder: &Path) -> Result<Variables> {
    let metadata_path = folder.join(FFIZER_DATASTORE_DIRNAME).join(OPTIONS_FILENAME);
    let variables = if metadata_path.exists() {
        let persisted: PersistedOptions =
            { serde_yaml::from_reader(std::fs::File::open(metadata_path)?)? };

        Variables::try_from(persisted.variables)?
    } else {
        Variables::default()
    };
    Ok(variables)
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use similar_asserts::assert_eq;
    use std::path::PathBuf;

    use super::*;
    use crate::cli_opt::ApplyOpts;
    use crate::tests::new_ctx_from;
    use crate::Ctx;
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
}
