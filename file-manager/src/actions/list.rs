use camino::{Utf8Path, Utf8PathBuf};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::{lister::Lister, DirEntry, DirIter, ListerOptions};

pub trait PathList {
    fn list(&self) -> ListBuilder;
}

impl PathList for Utf8Path {
    fn list(&self) -> ListBuilder {
        ListBuilder::new(self.to_path_buf())
    }
}

pub struct ListBuilder {
    lister: ListerOptions,
}

impl Deref for ListBuilder {
    type Target = ListerOptions;
    fn deref(&self) -> &Self::Target {
        &self.lister
    }
}
impl DerefMut for ListBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lister
    }
}

impl Lister for ListBuilder {}

impl ListBuilder {
    fn new(path: Utf8PathBuf) -> Self {
        Self {
            lister: ListerOptions::new(path),
        }
    }

    pub fn paths(self) -> crate::Result<Vec<Utf8PathBuf>> {
        Ok(self
            .into_iter()
            .map(|entry| entry.map(|e| e.relative_path().to_path_buf()))
            .collect::<crate::Result<Vec<Utf8PathBuf>>>()?)
    }

    pub fn collect<F, T>(self, op: F) -> crate::Result<Vec<T>>
    where
        F: Fn(DirEntry) -> T,
    {
        Ok(self
            .into_iter()
            .map(|entry| entry.map(|e| op(e)))
            .collect::<crate::Result<Vec<T>>>()?)
    }

    #[cfg(feature = "ascii")]
    pub fn ascii(self) -> crate::Result<String> {
        let tree = list_ascii(&self.lister.path).map_err(|source| {
            crate::FileManagerError::new(
                format!("Could not create ascii listing of {}", self.lister.path),
                source,
            )
        })?;
        Ok(tree.to_string())
    }
}

impl IntoIterator for ListBuilder {
    type Item = crate::Result<DirEntry>;
    type IntoIter = DirIter;

    fn into_iter(self) -> Self::IntoIter {
        self.lister.into_iter()
    }
}

#[cfg(feature = "ascii")]
fn list_ascii(path: impl AsRef<Path>) -> Result<crate::helpers::Tree, anyhow::Error> {
    use crate::helpers::{PathExt, Tree};
    use anyhow::bail;

    let path = path.as_ref();
    let Some(name) = path.file_name() else {
        bail!("Could not get file name from path {path:?}");
    };
    let name = name.to_string_lossy().to_string();
    if path.is_dir() {
        let entries = path.fm_read_dir()?;

        let mut content = Vec::new();

        for entry in entries {
            let Ok(entry) = entry else {
              panic!("")
            };
            let path = entry.path();
            content.push(list_ascii(path)?);
        }
        Ok(Tree::node(name, content))
    } else {
        Ok(Tree::leaf(name))
    }
}
