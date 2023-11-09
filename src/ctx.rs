use crate::cli_opt::*;
use crate::error::*;
use crate::variables::Variables;
use crate::SourceLoc;
use crate::SourceUri;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Ctx {
    pub cmd_opt: ApplyOpts,
}

pub(crate) const FFIZER_DATASTORE_DIRNAME: &str = ".ffizer";
const OPTIONS_FILENAME: &str = "options.yaml";
const VERSION_FILENAME: &str = "version.txt";

#[derive(Debug, Serialize, Deserialize)]
struct PersistedOptions {
    variables: Vec<PersistedVariable>,
    srcs: BTreeMap<String, PersistedSrc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSrc {
    uri: PersistedUri,
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
            uri: value.uri.into(),
            rev: value.rev,
            subfolder: value.subfolder,
        }
    }
}

impl From<PersistedSrc> for SourceLoc {
    fn from(value: PersistedSrc) -> Self {
        SourceLoc {
            uri: value.uri.into(),
            rev: value.rev,
            subfolder: value.subfolder,
        }
    }
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

    let mut confirmed_variables = Variables::default();
    confirmed_variables.append(&mut get_cli_variables(ctx)?);
    let suggested_variables = get_saved_variables(ctx)?;

    Ok((default_variables, confirmed_variables, suggested_variables))
}

pub(crate) fn save_options(variables: &Variables, ctx: &Ctx) -> Result<()> {
    let ffizer_folder = ctx.cmd_opt.dst_folder.join(FFIZER_DATASTORE_DIRNAME);
    if !ffizer_folder.exists() {
        std::fs::create_dir(&ffizer_folder)?;
    }
    // Save ffizer version
    fs::write(
        ffizer_folder.join(VERSION_FILENAME),
        env!("CARGO_PKG_VERSION"),
    )?;

    // Save or update default variable values stored in datastore
    let mut variables_to_save = get_saved_variables(ctx)?;
    variables_to_save.append(&mut variables.clone()); // update already existing keys
    variables_to_save.retain(|k, _v| !k.starts_with("ffizer_"));

    let mut saved_srcs: BTreeMap<String, PersistedSrc> = get_saved_sources(ctx)?
        .into_iter()
        .map(|(k, loc)| (k, PersistedSrc::from(loc)))
        .collect();

    let new_key = [
        ctx.cmd_opt.src.uri.raw.clone(),
        ctx.cmd_opt
            .src
            .subfolder
            .clone()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    ]
    .join(":");
    saved_srcs.insert(new_key, PersistedSrc::from(ctx.cmd_opt.src.clone()));

    let persisted_options = PersistedOptions {
        variables: variables_to_save.into(),
        srcs: saved_srcs,
    };
    serde_yaml::to_writer(
        std::fs::File::create(ffizer_folder.join(OPTIONS_FILENAME))?,
        &persisted_options,
    )?;
    Ok(())
}

pub fn get_saved_sources(ctx: &Ctx) -> Result<BTreeMap<String, SourceLoc>> {
    let metadata_path = ctx
        .cmd_opt
        .dst_folder
        .join(FFIZER_DATASTORE_DIRNAME)
        .join(OPTIONS_FILENAME);
    let sources = if metadata_path.exists() {
        let persisted: PersistedOptions =
            { serde_yaml::from_reader(std::fs::File::open(metadata_path)?)? };

        persisted
            .srcs
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    } else {
        BTreeMap::default()
    };
    Ok(sources)
}

pub fn get_saved_variables(ctx: &Ctx) -> Result<Variables> {
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

pub fn get_cli_variables(ctx: &Ctx) -> Result<Variables> {
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
pub(crate) mod tests {
    use super::*;
    pub use crate::cli_opt::*;
    use crate::tests::new_ctx_from;
    use tempfile::TempDir;

    fn new_variables_for_test() -> Variables {
        let mut variables = Variables::default();
        variables.insert("prj", "myprj").expect("insert prj");
        variables.insert("base", "remote").expect("insert base");
        variables
    }

    #[test]
    fn test_save_load_metadata() {
        let tmp_dir = TempDir::new().expect("create a temp dir");

        let ctx = new_ctx_from(tmp_dir.into_path());

        let variables = new_variables_for_test();

        let mut variables_with_ffizer = variables.clone();
        variables_with_ffizer
            .insert("ffizer_version", "0.0.0")
            .unwrap();

        save_options(&variables_with_ffizer, &ctx).unwrap();
        let saved_variables = get_saved_variables(&ctx).unwrap();
        assert_eq!(saved_variables, variables);
    }
}
