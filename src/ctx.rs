use crate::cli_opt::*;
use crate::error::*;
use crate::variables::Variables;
use std::io::Write;

#[derive(Debug, Clone, Default)]
pub struct Ctx {
    pub cmd_opt: ApplyOpts,
}

pub const FFIZER_DATASTORE_DIRNAME: &str = ".ffizer.d";

#[derive(Debug, Serialize, Deserialize)]
struct PersistedVariables {
    variables: Vec<SavedVariable>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedVariable {
    name: String,
    default_value: serde_yaml::Value,
}

impl TryFrom<PersistedVariables> for Variables {
    type Error = crate::Error;
    fn try_from(persisted: PersistedVariables) -> Result<Self> {
        let mut out = Variables::default();
        for saved_var in persisted.variables {
            out.insert(saved_var.name, saved_var.default_value)?;
        }
        Ok(out)
    }
}

impl From<Variables> for PersistedVariables {
    fn from(variables: Variables) -> Self {
        let formatted_variables = variables
            .tree()
            .iter()
            .map(|(k, v)| SavedVariable {
                name: k.into(),
                default_value: v.clone(),
            })
            .collect::<Vec<SavedVariable>>();
        PersistedVariables {
            variables: formatted_variables,
        }
    }
}

pub fn extract_variables(ctx: &Ctx) -> Result<(Variables, Variables)> {
    let mut confirmed_variables = Variables::default();
    confirmed_variables.insert(
        "ffizer_dst_folder",
        ctx.cmd_opt
            .dst_folder
            .to_str()
            .expect("dst_folder to converted via to_str"),
    )?;
    confirmed_variables.insert("ffizer_src_uri", ctx.cmd_opt.src.uri.raw.clone())?;
    confirmed_variables.insert("ffizer_src_rev", ctx.cmd_opt.src.rev.clone())?;
    confirmed_variables.insert("ffizer_src_subfolder", ctx.cmd_opt.src.subfolder.clone())?;
    confirmed_variables.insert("ffizer_version", env!("CARGO_PKG_VERSION"))?;

    confirmed_variables.append(&mut get_cli_variables(ctx)?);
    let suggested_variables = get_saved_variables(ctx)?;

    Ok((confirmed_variables, suggested_variables))
}

pub fn save_metadata(variables: &Variables, ctx: &Ctx) -> Result<()> {
    let ffizer_folder = ctx.cmd_opt.dst_folder.join(FFIZER_DATASTORE_DIRNAME);
    if !ffizer_folder.exists() {
        std::fs::create_dir(&ffizer_folder)?;
    }
    // Save ffizer version
    {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(ffizer_folder.join("version.txt"))?;
        if let Some(ffizer_version) = variables.get("ffizer_version").and_then(|x| x.as_str()) {
            write!(f, "{}", ffizer_version)?;
        }
    }

    // Save or update default variable values stored in datastore
    let mut variables_to_save = get_saved_variables(ctx)?;
    variables_to_save.append(&mut variables.clone()); // update already existing keys
    variables_to_save.retain(|k, _v| !k.starts_with("ffizer_"));

    let f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(ffizer_folder.join("variables.yaml"))?;
    serde_yaml::to_writer(f, &PersistedVariables::from(variables_to_save))?;
    Ok(())
}

pub fn get_saved_variables(ctx: &Ctx) -> Result<Variables> {
    let metadata_path = ctx
        .cmd_opt
        .dst_folder
        .join(FFIZER_DATASTORE_DIRNAME)
        .join("variables.yaml");
    let variables = if metadata_path.exists() {
        let persisted: PersistedVariables = {
            serde_yaml::from_reader(std::fs::OpenOptions::new().read(true).open(metadata_path)?)?
        };

        Variables::try_from(persisted)?
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
    use crate::PathBuf;
    use tempfile::TempDir;

    pub fn new_ctx_from<T: Into<PathBuf>>(dst: T) -> Ctx {
        Ctx {
            cmd_opt: ApplyOpts {
                dst_folder: dst.into(),
                ..Default::default()
            },
        }
    }

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

        save_metadata(&variables_with_ffizer, &ctx).unwrap();
        let saved_variables = get_saved_variables(&ctx).unwrap();
        assert_eq!(saved_variables, variables);
    }
}
