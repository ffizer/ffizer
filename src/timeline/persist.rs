use std::path::PathBuf;
use std::str::FromStr;

use crate::error::*;
use crate::variables::Variables;
use crate::SourceLoc;
use crate::SourceUri;

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedOptions {
    pub variables: Vec<PersistedVariable>,
    pub sources: Vec<PersistedSrc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedSrc {
    pub uri: String,
    pub rev: Option<String>,
    pub subfolder: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedVariable {
    pub name: String,
    pub default_value: serde_yaml::Value,
}

impl From<SourceLoc> for PersistedSrc {
    fn from(value: SourceLoc) -> Self {
        PersistedSrc {
            uri: value.uri.raw,
            rev: value.rev,
            subfolder: value.subfolder,
        }
    }
}

impl TryFrom<PersistedSrc> for SourceLoc {
    fn try_from(value: PersistedSrc) -> Result<Self> {
        Ok(SourceLoc {
            uri: SourceUri::from_str(&value.uri)?,
            rev: value.rev,
            subfolder: value.subfolder,
        })
    }
    type Error = crate::Error;
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
