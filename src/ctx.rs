use crate::cli_opt::*;
use crate::variables::Variables;
use crate::error::*;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Debug, Clone, Default)]
pub struct Ctx {
    pub cmd_opt: ApplyOpts,
}

pub const FFIZER_DATASTORE_DIRNAME: &str = ".ffizer.d";

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
    let formatted_variables = variables_to_save
        .tree()
        .iter()
        .filter(|(k, _v)| !k.starts_with("ffizer_"))
        .map(|(k, v)| {
            let mut map = Mapping::new();
            map.insert("key".into(), Value::String(k.into()));
            map.insert("value".into(), v.clone());
            map
        })
        .collect::<Vec<Mapping>>();

    let mut output_tree: BTreeMap<String, Vec<Mapping>> = BTreeMap::new();
    output_tree.insert("variables".to_string(), formatted_variables);

    let f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(ffizer_folder.join("variables.yaml"))?;
    serde_yaml::to_writer(f, &output_tree)?;
    Ok(())
}

pub fn get_saved_variables(ctx: &Ctx) -> Result<Variables> {
    let mut variables = Variables::default();
    let metadata_path = ctx
        .cmd_opt
        .dst_folder
        .join(FFIZER_DATASTORE_DIRNAME)
        .join("variables.yaml");
    if metadata_path.exists() {
        let metadata: Mapping = {
            let f = std::fs::OpenOptions::new().read(true).open(metadata_path)?;
            serde_yaml::from_reader::<_, Mapping>(f)?
        };

        let nodes = metadata
            .get("variables")
            .and_then(|v| v.as_sequence())
            .ok_or(Error::ConfigError {
                error: format!(
                    "Did not find a sequence at key 'variables' in config {:?}",
                    metadata
                ),
            })?;
        for node in nodes
            .iter()
            .map(|x| {
                x.as_mapping().ok_or(Error::ConfigError {
                    error: format!("Failed to parse node as a mapping in sequence {:?}", nodes),
                })
            })
            .collect::<Result<Vec<&Mapping>>>()?
        {
            let k = node
                .get("key")
                .and_then(|k| k.as_str())
                .ok_or(Error::ConfigError {
                    error: format!("Could not parse key 'key' in node {:?}", node),
                })?;
            let value = node.get("value").ok_or(Error::ConfigError {
                error: format!("Could not parse key 'value' in node {:?}", node),
            })?;
            variables.insert(k, value)?;
        }
    }
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
pub (crate) mod tests {
    use super::*;
    pub use crate::cli_opt::*;
    use tempfile::TempDir;
    use crate::PathBuf;

    const DST_FOLDER_STR: &str = "test/dst";
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

    fn new_variables_for_test() -> Variables {
        let mut variables = Variables::default();
        variables.insert("prj", "myprj").expect("insert prj");
        variables.insert("base", "remote").expect("insert base");
        variables
    }

    #[test]
    fn test_save_load_metadata() {
        let tmp_dir = TempDir::new().expect("create a temp dir");

        let mut ctx = new_ctx_for_test();
        ctx.cmd_opt.dst_folder = tmp_dir.into_path();

        let variables = new_variables_for_test();

        let mut variables_with_ffizer = variables.clone();
        variables_with_ffizer.insert("ffizer_version", "0.0.0").unwrap();

        save_metadata(&variables_with_ffizer, &ctx).unwrap();
        let saved_variables = get_saved_variables(&ctx).unwrap();
        assert_eq!(saved_variables, variables);
    }
}