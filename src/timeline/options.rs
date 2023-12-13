use crate::{Result,Error};
use crate::timeline::{PersistedOptions, PersistedVariable, PersistedSrc, key_from_loc};
use crate::timeline::{FFIZER_DATASTORE_DIRNAME,VERSION_FILENAME};
use crate::{SourceLoc, Variables, SourceUri};

use std::path::Path;
use std::str::FromStr;
use std::fs;

use pathdiff::diff_paths;

const OPTIONS_FILENAME: &str = "options.yaml";

pub(crate) fn load_options(folder: &Path) -> Result<PersistedOptions> {
    let metadata_path = folder.join(FFIZER_DATASTORE_DIRNAME).join(OPTIONS_FILENAME);
    if metadata_path.exists() {
        let options = serde_yaml::from_reader(std::fs::File::open(
            metadata_path,
        )?)?;
        Ok(options)
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
    
    let relative_source: SourceLoc;
    let source: &SourceLoc = {
        if source.uri.host.is_none() && !source.uri.path.is_absolute() { // uri is local and relative, we need to make it relative to dst_folder
            let local_path = source.uri
                .path
                .canonicalize()
                .map_err(|err| Error::CanonicalizePath {
                    path: source.uri.path.clone(),
                    source: err,
                })?;
            let relative_path = diff_paths(local_path, dst_folder.canonicalize()?).unwrap();
            relative_source = SourceLoc {
                uri: SourceUri::from_str(&relative_path.to_string_lossy())?,
                ..source.clone()
            };
            &relative_source
        } else {
            source
        }
    };

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
