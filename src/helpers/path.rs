use std::{fs, path::Path};

use anyhow::{anyhow, Context};
use camino::{Utf8Path, Utf8PathBuf};

pub trait Utf8PathBufExt {
    fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Utf8PathBuf>;
}

impl Utf8PathBufExt for Utf8PathBuf {
    fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Utf8PathBuf> {
        Utf8PathBuf::from_path_buf(path.as_ref().to_path_buf())
            .map_err(|path| anyhow!("could not convert path {path:?} to Utf8PathBuf"))
    }
}
pub trait Utf8PathExt {
    fn remove_dir_all(&self) -> anyhow::Result<Utf8PathBuf>;
    fn create_dir(&self) -> anyhow::Result<Utf8PathBuf>;
    fn replace_dir(&self) -> anyhow::Result<Utf8PathBuf>;
}

impl Utf8PathExt for Utf8Path {
    fn remove_dir_all(&self) -> anyhow::Result<Utf8PathBuf> {
        fs::remove_dir_all(self).with_context(|| format!("could not remove dir {self:?}"))?;
        Ok(self.to_owned())
    }

    fn create_dir(&self) -> anyhow::Result<Utf8PathBuf> {
        fs::create_dir(self).with_context(|| format!("could not create dir {self:?}"))?;
        Ok(self.to_owned())
    }

    fn replace_dir(&self) -> anyhow::Result<Utf8PathBuf> {
        if self.exists() {
            self.remove_dir_all()?;
        }
        self.create_dir()?;
        Ok(self.to_owned())
    }
}

pub trait PathExt {
    fn fm_read_dir(&self) -> anyhow::Result<fs::ReadDir>;

    fn to_utf8_path_buf(&self) -> anyhow::Result<Utf8PathBuf>;
}

impl PathExt for Path {
    fn fm_read_dir(&self) -> anyhow::Result<fs::ReadDir> {
        fs::read_dir(self).with_context(|| format!("could not read dir {self:?}"))
    }

    fn to_utf8_path_buf(&self) -> anyhow::Result<Utf8PathBuf> {
        Utf8PathBuf::from_path(self)
            .with_context(|| format!("could not convert path {self:?} to Utf8PathBuf"))
    }
}
