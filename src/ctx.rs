use super::Ctx;
use crate::error::*;
use crate::variables::Variables;
use crate::SourceLoc;
use crate::SourceUri;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub(crate) const FFIZER_DATASTORE_DIRNAME: &str = ".ffizer";
const OPTIONS_FILENAME: &str = "options.yaml";
const VERSION_FILENAME: &str = "version.txt";

#[derive(Debug, Serialize, Deserialize)]
struct PersistedOptions {
    variables: Vec<PersistedVariable>,
    sources: Vec<PersistedSrc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSrc {
    uri: String,
    rev: Option<String>,
    subfolder: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedUri {
    raw: String,
    path: PathBuf,
    host: Option<String>,
}

impl From<SourceUri> for PersistedUri {
    fn from(value: SourceUri) -> Self {
        PersistedUri {
            raw: value.raw,
            path: value.path,
            host: value.host,
        }
    }
}

impl From<PersistedUri> for SourceUri {
    fn from(value: PersistedUri) -> Self {
        SourceUri {
            raw: value.raw,
            path: value.path,
            host: value.host,
        }
    }
}

impl From<SourceLoc> for PersistedSrc {
    fn from(value: SourceLoc) -> Self {
        PersistedSrc {
            uri: value.uri.raw,
            rev: value.rev,
            subfolder: value.subfolder,
        }
    }
}

impl TryFrom<PersistedSrc> for SourceLoc {
    fn try_from(value: PersistedSrc) -> Result<Self> {
        Ok(SourceLoc {
            uri: SourceUri::from_str(&value.uri)?,
            rev: value.rev,
            subfolder: value.subfolder,
        })
    }
    type Error = crate::Error;
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedVariable {
    name: String,
    default_value: serde_yaml::Value,
}

impl TryFrom<Vec<PersistedVariable>> for Variables {
    type Error = crate::Error;
    fn try_from(persisted: Vec<PersistedVariable>) -> Result<Self> {
        let mut out = Variables::default();
        for saved_var in persisted {
            out.insert(saved_var.name, saved_var.default_value)?;
        }
        Ok(out)
    }
}

impl From<Variables> for Vec<PersistedVariable> {
    fn from(variables: Variables) -> Self {
        variables
            .tree()
            .iter()
            .map(|(k, v)| PersistedVariable {
                name: k.into(),
                default_value: v.clone(),
            })
            .collect::<Vec<PersistedVariable>>()
    }
}

pub(crate) fn extract_variables(ctx: &Ctx) -> Result<(Variables, Variables, Variables)> {
    let mut default_variables = Variables::default();
    default_variables.insert(
        "ffizer_dst_folder",
        ctx.cmd_opt
            .dst_folder
            .to_str()
            .expect("dst_folder to converted via to_str"),
    )?;
    default_variables.insert("ffizer_src_uri", ctx.cmd_opt.src.uri.raw.clone())?;
    default_variables.insert("ffizer_src_rev", ctx.cmd_opt.src.rev.clone())?;
    default_variables.insert("ffizer_src_subfolder", ctx.cmd_opt.src.subfolder.clone())?;
    default_variables.insert("ffizer_version", env!("CARGO_PKG_VERSION"))?;

    let confirmed_variables = get_cli_variables(ctx)?;
    let suggested_variables = get_saved_variables(ctx)?;

    Ok((default_variables, confirmed_variables, suggested_variables))
}

fn key_from_uri(uri: &SourceUri) -> String {
    format!(
        "{}:{}",
        uri.host.clone().unwrap_or("".to_string()),
        uri.path.to_string_lossy()
    )
}

fn key_from_loc(source: &SourceLoc) -> String {
    let uri_key = key_from_uri(&source.uri);
    if let Some(p) = source.subfolder.clone() {
        format!("{}@{}", p.to_string_lossy().into_owned(), uri_key)
    } else {
        uri_key
    }
}

pub(crate) fn save_options(variables: &Variables, ctx: &Ctx) -> Result<()> {
    // Save or update default variable values stored in datastore
    let mut variables_to_save = get_saved_variables(ctx)?;
    variables_to_save.append(&mut variables.clone()); // update already existing keys
    variables_to_save.retain(|k, _v| !k.starts_with("ffizer_"));

    let new_key: String = key_from_loc(&ctx.cmd_opt.src);

    let mut saved_srcs: Vec<PersistedSrc> = get_saved_sources(&ctx.cmd_opt.dst_folder)?
        .into_iter()
        .filter(|loc| key_from_loc(loc) != new_key)
        .map(|loc| loc.into())
        .collect();

    saved_srcs.push(ctx.cmd_opt.src.clone().into());

    let persisted_options = PersistedOptions {
        variables: variables_to_save.into(),
        sources: saved_srcs,
    };

    let ffizer_folder = ctx.cmd_opt.dst_folder.join(FFIZER_DATASTORE_DIRNAME);
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

fn get_saved_variables(ctx: &Ctx) -> Result<Variables> {
    let metadata_path = ctx
        .cmd_opt
        .dst_folder
        .join(FFIZER_DATASTORE_DIRNAME)
        .join(OPTIONS_FILENAME);
    let variables = if metadata_path.exists() {
        let persisted: PersistedOptions =
            { serde_yaml::from_reader(std::fs::File::open(metadata_path)?)? };

        Variables::try_from(persisted.variables)?
    } else {
        Variables::default()
    };
    Ok(variables)
}

fn get_cli_variables(ctx: &Ctx) -> Result<Variables> {
    let mut variables = Variables::default();
    ctx.cmd_opt
        .key_value
        .iter()
        .map(|(k, v)| {
            let v = match v.to_lowercase().trim() {
                "true" | "y" | "yes" => "true",
                "false" | "n" | "no" => "false",
                _ => v.trim(),
            };
            variables.insert(k, Variables::value_from_str(v)?)
        })
        .collect::<Result<Vec<()>>>()?;
    Ok(variables)
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use similar_asserts::assert_eq;
    use std::path::Path;
    use std::str::FromStr;

    use super::*;
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

        save_options(&variables_with_ffizer, &ctx).unwrap();
        let saved_variables = get_saved_variables(&ctx).unwrap();
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

            save_options(&variables, &ctx_1).unwrap();

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

            save_options(&variables, &ctx_1).unwrap();
            save_options(&variables, &ctx_2).unwrap();

            let saved_sources = get_saved_sources(&ctx_1.cmd_opt.dst_folder).unwrap();

            let expected = vec![source_1, source_2];
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

            save_options(&variables, &ctx_1).unwrap();
            save_options(&variables, &ctx_2).unwrap();

            let saved_sources = get_saved_sources(&ctx_1.cmd_opt.dst_folder).unwrap();

            let expected = vec![source_2];
            assert_eq!(expected, saved_sources);
        }
    }
}
