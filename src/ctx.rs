use super::Ctx;
use crate::error::*;
use crate::timeline::get_saved_variables;
use crate::variables::Variables;

pub(crate) struct VariablesFromCtx {
    pub src: Variables,
    pub cli: Variables,
    pub saved: Variables,
}

pub(crate) fn extract_variables(ctx: &Ctx) -> Result<VariablesFromCtx> {
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
    let suggested_variables = get_saved_variables(&ctx.cmd_opt.dst_folder)?;

    Ok(VariablesFromCtx {
        src: default_variables,
        cli: confirmed_variables,
        saved: suggested_variables,
    })
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
