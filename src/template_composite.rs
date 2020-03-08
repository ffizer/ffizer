use crate::files;
use crate::graph::Graph;
use crate::scripts::Script;
use crate::source_file::SourceFile;
use crate::source_loc::SourceLoc;
use crate::template_cfg::TemplateCfg;
use crate::transform_values::TransformsValues;
use crate::variable_def::VariableDef;
use crate::Ctx;
use crate::Result;
use crate::Variables;
use handlebars_misc_helpers::new_hbs;
use slog::{debug, warn};
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct TemplateLayer {
    order: usize,
    loc: SourceLoc,
    cfg: TemplateCfg,
}

impl TransformsValues for TemplateLayer {
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let cfg = self.cfg.transforms_values(render)?;
        Ok(TemplateLayer {
            order: self.order.clone(),
            loc: self.loc.clone(),
            cfg,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TemplateComposite {
    layers: Vec<TemplateLayer>,
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
            .enumerate()
            .map(|(i, k)| {
                let v = templates.get(&k).expect("should exist").clone();
                TemplateLayer {
                    order: i,
                    loc: k,
                    cfg: v,
                }
            })
            .collect::<Vec<_>>();
        debug!(ctx.logger, "templates"; "layers" => ?layers);
        Ok(TemplateComposite { layers })
    }

    pub fn variables(&self) -> Vec<VariableDef> {
        let mut back = vec![];
        let mut names = HashSet::new();
        for layer in &self.layers {
            for variable in &layer.cfg.variables {
                if !names.contains(&variable.name) {
                    names.insert(variable.name.clone());
                    back.push(variable.clone());
                }
            }
        }
        back
    }

    pub fn find_sourcefiles(&self) -> Result<Vec<SourceFile>> {
        let mut back = vec![];
        for layer in &self.layers {
            let ignores = &layer.cfg.ignores;
            let template_dir = if layer.cfg.use_template_dir {
                "template"
            } else {
                ""
            };
            let path = layer.loc.as_local_path()?.join(template_dir);
            for childpath in files::find_childpaths(path, ignores) {
                back.push(SourceFile::from((childpath, layer.order)));
            }
        }
        Ok(back)
    }

    pub fn scripts<'a>(&'a self) -> impl Iterator<Item = (&'a SourceLoc, &'a Vec<Script>)> {
        self.layers
            .iter()
            .map(|t| (&t.loc, &t.cfg.scripts))
            .into_iter()
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
        variables_children.insert("ffizer_src_uri", src.uri.raw.clone())?;
        variables_children.insert("ffizer_src_rev", src.rev.clone())?;
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

impl TransformsValues for TemplateComposite {
    /// transforms ignore, imports
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let layers = self.layers.transforms_values(render)?;
        Ok(TemplateComposite { layers })
    }
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
