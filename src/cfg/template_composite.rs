use super::template_cfg::TemplateCfg;
use super::transform_values::TransformsValues;
use super::variable_cfg::VariableCfg;
use crate::files;
use crate::graph::Graph;
use crate::scripts::Script;
use crate::source_file::SourceFile;
use crate::source_loc::SourceLoc;
use crate::Result;
use crate::Variables;
use handlebars_misc_helpers::new_hbs;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::{debug, instrument, span, warn, Level};
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
            order: self.order,
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
        variables: &Variables,
        offline: bool,
        src: &SourceLoc,
    ) -> Result<TemplateComposite> {
        let mut templates = HashMap::new();
        deep_download(variables, offline, src, &mut templates)?;
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
        debug!(?layers);
        Ok(TemplateComposite { layers })
    }

    pub fn find_variablecfgs(&self) -> Result<Vec<VariableCfg>> {
        let mut back = vec![];
        let mut names = HashSet::new();
        for layer in &self.layers {
            let _span_ = span!(Level::DEBUG, "find_variablecfgs", layer = ?layer).entered();
            for variable in layer.cfg.variables.clone() {
                if !names.contains(&variable.name) {
                    names.insert(variable.name.clone());
                    back.push(variable.clone());
                }
            }
        }
        Ok(back)
    }

    pub fn find_sourcefiles(&self) -> Result<Vec<SourceFile>> {
        let mut back = vec![];
        for layer in &self.layers {
            let _span_ = span!(Level::DEBUG, "find_sourcefiles", layer = ?layer).entered();
            let ignores = &layer.cfg.find_ignores()?;
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

    pub fn find_scripts(&self) -> Result<Vec<(&SourceLoc, Vec<Script>)>> {
        self.layers
            .iter()
            .map(|t| t.cfg.find_scripts().map(|l| (&t.loc, l)))
            .collect()
    }
}

impl Graph for HashMap<SourceLoc, TemplateCfg> {
    type K = SourceLoc;
    type V = TemplateCfg;
    fn find_node(&self, k: &Self::K) -> Option<&Self::V> {
        self.get(k)
    }
    fn find_edges_direct(&self, v: &Self::V) -> Vec<Self::K> {
        v.find_sourcelocs()
            .expect("TODO find_sourcelocs without error")
    }
}

//struct Template;
#[instrument(skip(variables, templates))]
fn deep_download(
    variables: &Variables,
    offline: bool,
    src: &SourceLoc,
    templates: &mut HashMap<SourceLoc, TemplateCfg>,
) -> Result<()> {
    if !templates.contains_key(src) {
        let template_base_path = &src.download(offline)?;
        // update cfg with variables defined by user
        let template_cfg = TemplateCfg::from_template_folder(&template_base_path)?;
        // update cfg with variables defined by cli (use to update default_value)
        let mut variables_children = variables.clone();
        variables_children.insert("ffizer_src_uri", src.uri.raw.clone())?;
        variables_children.insert("ffizer_src_rev", src.rev.clone())?;
        //variables_children.insert("ffizer_src_subfolder".to_owned(), src.subfolder.clone());
        let template_cfg_for_imports =
            render_imports_only(&template_cfg, &variables_children, false)?;
        let children = template_cfg_for_imports.find_sourcelocs()?;
        //WARN: Do insert a rendered templates because the value of are not yet defined
        templates.insert(src.clone(), template_cfg_for_imports);
        for child in children {
            deep_download(&variables_children, offline, &child, templates)?;
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

pub(crate) fn render_composite(
    template_composite: &TemplateComposite,
    variables: &Variables,
    log_warning: bool,
) -> Result<TemplateComposite> {
    let handlebars = new_hbs();
    let render = |v: &str| {
        let r = handlebars.render_template(v, variables);
        match r {
            Ok(s) => s,
            Err(e) => {
                if log_warning {
                    warn!(input = ?v, error = ?e, "failed to convert")
                }
                v.into()
            }
        }
    };
    template_composite.transforms_values(&render)
}

fn render_imports_only(
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
                    warn!(input = ?v, error = ?e, "failed to convert")
                }
                v.into()
            }
        }
    };
    let variables = template_cfg.variables.clone();
    let ignores = template_cfg.ignores.clone();
    let imports = template_cfg.imports.transforms_values(&render)?;
    let scripts = template_cfg.scripts.clone();
    Ok(TemplateCfg {
        variables,
        ignores,
        imports,
        scripts,
        use_template_dir: template_cfg.use_template_dir,
    })
}
