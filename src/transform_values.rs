use failure::format_err;
use failure::Error;
use std::path::PathBuf;

pub trait TransformsValues {
    fn transforms_values<F>(&self, render: &F) -> Result<Self, Error>
    where
        F: Fn(&str) -> String,
        Self: Sized;
}

impl TransformsValues for PathBuf {
    fn transforms_values<F>(&self, render: &F) -> Result<Self, Error>
    where
        F: Fn(&str) -> String,
    {
        self.to_str()
            .ok_or_else(|| format_err!("failed to stringify path"))
            .map(|s| PathBuf::from(render(s)))
    }
}

impl<T> TransformsValues for Vec<T>
where
    T: TransformsValues,
{
    fn transforms_values<F>(&self, render: &F) -> Result<Self, Error>
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
