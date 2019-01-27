use crate::files;
use crate::graph::Graph;
use crate::hbs;
use crate::source_loc::SourceLoc;
use crate::template_cfg::TemplateCfg;
use crate::template_cfg::Variable;
use crate::ChildPath;
use crate::Ctx;
use crate::Variables;
use failure::Error;
use slog::warn;
use std::collections::BTreeMap;
use std::collections::HashSet;

pub struct TemplateComposite {
    layers: Vec<(SourceLoc, TemplateCfg)>,
}

impl TemplateComposite {
    pub fn from_src(
        ctx: &Ctx,
        variables: &Variables,
        offline: bool,
        src: &SourceLoc,
    ) -> Result<TemplateComposite, Error> {
        let mut templates = BTreeMap::new();
        deep_download(ctx, variables, offline, src, &mut templates)?;
        let layers = templates
            .find_edges_ordered_by_depth(src)
            .into_iter()
            .map(|k| {
                let v = templates.get(&k).expect("should exist").clone();
                (k, v)
            })
            .collect::<Vec<_>>();
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

    pub fn find_childpaths(&self) -> Result<Vec<ChildPath>, Error> {
        let mut back = vec![];
        let mut relatives = HashSet::new();
        for layer in &self.layers {
            let path = layer.0.as_local_path()?;
            let ignores = &layer.1.ignores;
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

impl Graph for BTreeMap<SourceLoc, TemplateCfg> {
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
    templates: &mut BTreeMap<SourceLoc, TemplateCfg>,
) -> Result<(), Error> {
    if !templates.contains_key(src) {
        let template_base_path = &src.download(offline)?;
        // update cfg with variables defined by user
        let mut template_cfg = TemplateCfg::from_template_folder(&template_base_path)?;
        // update cfg with variables defined by cli (use to update default_value)
        template_cfg = render_cfg(&ctx, &template_cfg, &variables, false)?;
        let children = template_cfg.imports.clone();
        templates.insert(src.clone(), template_cfg);
        for child in children {
            deep_download(ctx, variables, offline, &child, templates)?;
        }
    }
    Ok(())
}

fn render_cfg(
    ctx: &Ctx,
    template_cfg: &TemplateCfg,
    variables: &Variables,
    log_warning: bool,
) -> Result<TemplateCfg, Error> {
    let handlebars = hbs::new_hbs()?;
    template_cfg.transforms_values(|v| {
        let r = handlebars.render_template(v, variables);
        match r {
            Ok(s) => s,
            Err(e) => {
                if log_warning { warn!(ctx.logger, "failed to convert"; "input" => v, "error" => format!("{:?}", e))}
                v.into()
            }
        }
    })
}
