use crate::files;
use crate::graph::Graph;
use crate::source_loc::SourceLoc;
use crate::template_cfg::TemplateCfg;
use crate::template_cfg::Variable;
use crate::transform_values::TransformsValues;
use crate::ChildPath;
use crate::Ctx;
use crate::Result;
use crate::Variables;
use handlebars_misc_helpers::new_hbs;
use std::collections::HashMap;
use std::collections::HashSet;
use slog::{debug,warn};

pub struct TemplateComposite {
    layers: Vec<(SourceLoc, TemplateCfg)>,
}

impl TemplateComposite {
    pub fn from_src(
        ctx: &Ctx,
        variables: &Variables,
        offline: bool,
        src: &SourceLoc,
    ) -> Result<TemplateComposite> {
        let mut templates = HashMap::new();
        deep_download(ctx, variables, offline, src, &mut templates)?;
        let layers = templates
            .find_edges_ordered_by_depth(src)
            .into_iter()
            .map(|k| {
                let v = templates.get(&k).expect("should exist").clone();
                (k, v)
            })
            .collect::<Vec<_>>();
        debug!(ctx.logger, "templates"; "layers" => ?layers.iter().map(|kv| &kv.0).collect::<Vec<_>>());
        Ok(TemplateComposite { layers })
    }

    pub fn variables(&self) -> Vec<Variable> {
        let mut back = vec![];
        let mut names = HashSet::new();
        for layer in &self.layers {
            for variable in &layer.1.variables {
                if !names.contains(&variable.name) {
                    names.insert(variable.name.clone());
                    back.push(variable.clone());
                }
            }
        }
        back
    }

    pub fn find_childpaths(&self) -> Result<Vec<ChildPath>> {
        let mut back = vec![];
        let mut relatives = HashSet::new();
        for layer in &self.layers {
            let ignores = &layer.1.ignores;
            let template_dir = if layer.1.use_template_dir {
                "template"
            } else {
                ""
            };
            let path = layer.0.as_local_path()?.join(template_dir);
            for childpath in files::find_childpaths(path, ignores) {
                if !relatives.contains(&childpath.relative) {
                    relatives.insert(childpath.relative.clone());
                    back.push(childpath);
                }
            }
        }
        Ok(back)
    }
}

impl Graph for HashMap<SourceLoc, TemplateCfg> {
    type K = SourceLoc;
    type V = TemplateCfg;
    fn find_node(&self, k: &Self::K) -> Option<&Self::V> {
        self.get(k)
    }
    fn find_edges_direct(&self, v: &Self::V) -> Vec<Self::K> {
        v.imports.clone()
    }
}

//struct Template;
fn deep_download(
    ctx: &Ctx,
    variables: &Variables,
    offline: bool,
    src: &SourceLoc,
    templates: &mut HashMap<SourceLoc, TemplateCfg>,
) -> Result<()> {
    if !templates.contains_key(src) {
        let template_base_path = &src.download(ctx, offline)?;
        // update cfg with variables defined by user
        let mut template_cfg = TemplateCfg::from_template_folder(&template_base_path)?;
        // update cfg with variables defined by cli (use to update default_value)
        let mut variables_children = variables.clone();
        variables_children.insert("ffizer_src_uri".to_owned(), src.uri.raw.clone());
        variables_children.insert("ffizer_src_rev".to_owned(), src.rev.clone());
        //variables_children.insert("ffizer_src_subfolder".to_owned(), src.subfolder.clone());
        template_cfg = render_cfg(&ctx, &template_cfg, &variables_children, false)?;
        let children = template_cfg.imports.clone();
        templates.insert(src.clone(), template_cfg);
        for child in children {
            deep_download(ctx, &variables_children, offline, &child, templates)?;
        }
    }
    Ok(())
}

fn render_cfg(
    ctx: &Ctx,
    template_cfg: &TemplateCfg,
    variables: &Variables,
    log_warning: bool,
) -> Result<TemplateCfg> {
    let handlebars = new_hbs();
    let render = |v: &str| {
        let r = handlebars.render_template(v, variables);
        match r {
            Ok(s) => s,
            Err(e) => {
                if log_warning {
                    warn!(ctx.logger, "failed to convert"; "input" => ?v, "error" => ?e)
                }
                v.into()
            }
        }
    };
    template_cfg.transforms_values(&render)
}
