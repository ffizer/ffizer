use crate::cli_opt::*;
use crate::error::*;
use crate::variables::Variables;
use std::fs;

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
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedVariable {
    name: String,
    default_value: serde_yaml::Value,
}

impl TryFrom<PersistedOptions> for Variables {
    type Error = crate::Error;
    fn try_from(persisted: PersistedOptions) -> Result<Self> {
        let mut out = Variables::default();
        for saved_var in persisted.variables {
            out.insert(saved_var.name, saved_var.default_value)?;
        }
        Ok(out)
    }
}

impl From<Variables> for PersistedOptions {
    fn from(variables: Variables) -> Self {
        let formatted_variables = variables
            .tree()
            .iter()
            .map(|(k, v)| PersistedVariable {
                name: k.into(),
                default_value: v.clone(),
            })
            .collect::<Vec<PersistedVariable>>();
        PersistedOptions {
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

pub fn save_options(variables: &Variables, ctx: &Ctx) -> Result<()> {
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
    let f = std::fs::File::create(ffizer_folder.join(OPTIONS_FILENAME))?;
    serde_yaml::to_writer(f, &PersistedOptions::from(variables_to_save))?;
    Ok(())
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
