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
            .ok_or(crate::Error::Any {
                msg: "failed to stringify path".to_owned(),
            })
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
