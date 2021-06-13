use crate::Result;
use std::path::PathBuf;

pub trait TransformsValues {
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
        Self: Sized;
}

impl TransformsValues for String {
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let b = render(self);
        Ok(b)
    }
}

impl TransformsValues for PathBuf {
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        self.to_str()
            .ok_or_else(|| crate::Error::Unknown("failed to stringify path".to_owned()))
            .map(|s| PathBuf::from(render(s)))
    }
}

impl<T> TransformsValues for Vec<T>
where
    T: TransformsValues,
{
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        let mut b = Vec::with_capacity(self.len());
        for i in self {
            b.push(i.transforms_values(render)?);
        }
        Ok(b)
    }
}

impl<T> TransformsValues for Option<T>
where
    T: TransformsValues,
{
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        match self {
            None => Ok(None),
            Some(v) => Ok(Some(v.transforms_values(render)?)),
        }
    }
}

impl TransformsValues for serde_yaml::Value {
    fn transforms_values<F>(&self, render: &F) -> Result<Self>
    where
        F: Fn(&str) -> String,
    {
        match self {
            serde_yaml::Value::String(ref v) => {
                Ok(serde_yaml::Value::String(v.transforms_values(render)?))
            }
            serde_yaml::Value::Sequence(ref s) => {
                Ok(serde_yaml::Value::Sequence(s.transforms_values(render)?))
            }
            serde_yaml::Value::Mapping(ref m) => {
                let mut expected = serde_yaml::Mapping::new();
                for (k, v) in m {
                    expected.insert(k.transforms_values(render)?, v.transforms_values(render)?);
                }
                Ok(serde_yaml::Value::Mapping(expected))
            }
            _ => Ok(self.clone()),
        }
    }
}
