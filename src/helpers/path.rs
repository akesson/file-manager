use std::{fs, path::Path};

use anyhow::{anyhow, Context};
use camino::Utf8PathBuf;

pub trait Utf8PathExt {
    fn from_path(path: impl AsRef<Path>) -> Result<Utf8PathBuf, anyhow::Error>;
}

impl Utf8PathExt for Utf8PathBuf {
    fn from_path(path: impl AsRef<Path>) -> Result<Utf8PathBuf, anyhow::Error> {
        Utf8PathBuf::from_path_buf(path.as_ref().to_path_buf())
            .map_err(|path| anyhow!("could not convert path {path:?} to Utf8PathBuf"))
    }
}

pub trait PathExt {
    fn fm_read_dir(&self) -> Result<fs::ReadDir, anyhow::Error>;

    fn to_utf8_path_buf(&self) -> Result<Utf8PathBuf, anyhow::Error>;
}

impl PathExt for Path {
    fn fm_read_dir(&self) -> Result<fs::ReadDir, anyhow::Error> {
        fs::read_dir(self).with_context(|| format!("could not read dir {self:?}"))
    }

    fn to_utf8_path_buf(&self) -> Result<Utf8PathBuf, anyhow::Error> {
        Utf8PathBuf::from_path(self)
            .with_context(|| format!("could not convert path {self:?} to Utf8PathBuf"))
    }
}
