use super::Ctx;
use crate::error::Result;
use crate::timeline::get_saved_variables;
use crate::variables::Variables;

pub(crate) struct VariablesFromCtx {
    pub src: Variables,
    pub cli: Variables,
    pub saved: Variables,
}

pub(crate) fn extract_variables(ctx: &Ctx) -> Result<VariablesFromCtx> {
    let mut ctx_variables = Variables::default();
    ctx_variables.insert(
        "ffizer_dst_folder",
        ctx.cmd_opt
            .dst_folder
            .to_str()
            .expect("dst_folder to converted via to_str"),
    )?;
    ctx_variables.insert("ffizer_src_uri", ctx.cmd_opt.src.uri.raw.clone())?;
    ctx_variables.insert("ffizer_src_rev", ctx.cmd_opt.src.rev.clone())?;
    ctx_variables.insert("ffizer_src_subfolder", ctx.cmd_opt.src.subfolder.clone())?;
    ctx_variables.insert("ffizer_version", env!("CARGO_PKG_VERSION"))?;

    Ok(VariablesFromCtx {
        src: ctx_variables,
        cli: get_cli_variables(ctx)?,
        saved: get_saved_variables(&ctx.cmd_opt.dst_folder)?,
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
