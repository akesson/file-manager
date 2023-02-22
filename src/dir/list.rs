use camino::{Utf8Path, Utf8PathBuf};

use crate::{
    build::{Build, DirOptions, DirOptionsMut, FilterOptionMut, FilterOptions, PathNew},
    helpers::DirIter,
};

pub trait DirList {
    fn dir_list(&self) -> Build<ListOptions>;
}

impl DirList for Utf8Path {
    fn dir_list(&self) -> Build<ListOptions> {
        Build::new(&self)
    }
}
pub struct ListOptions {
    dir_options: DirOptions,
    filter_options: FilterOptions,
}

impl PathNew for ListOptions {
    fn new(path: impl AsRef<Utf8Path>) -> Self {
        ListOptions {
            dir_options: DirOptions::new(path.as_ref()),
            filter_options: FilterOptions::default(),
        }
    }
}

impl DirOptionsMut for ListOptions {
    fn dir_options(&mut self) -> &mut DirOptions {
        &mut self.dir_options
    }
}

impl FilterOptionMut for ListOptions {
    fn filter_options(&mut self) -> &mut FilterOptions {
        &mut self.filter_options
    }
}

impl Build<ListOptions> {
    #[cfg(feature = "ascii")]
    pub fn ascii(self) -> crate::Result<String> {
        let tree = list_ascii(&self.path).map_err(|source| {
            crate::FileManagerError::new(
                format!("Could not create ascii listing of {}", self.path),
                source,
            )
        })?;
        Ok(tree.to_string())
    }

    pub fn paths(self) -> crate::Result<Vec<Utf8PathBuf>> {
        Ok(self
            .into_iter()
            .map(|entry| entry.map(|e| e.relative_path().to_path_buf()))
            .collect::<crate::Result<Vec<Utf8PathBuf>>>()?)
    }

    pub fn collect<F, T>(self, op: F) -> crate::Result<Vec<T>>
    where
        F: Fn(crate::helpers::DirEntry) -> T,
    {
        Ok(self
            .into_iter()
            .map(|entry| entry.map(|e| op(e)))
            .collect::<crate::Result<Vec<T>>>()?)
    }
}

impl IntoIterator for Build<ListOptions> {
    type Item = crate::Result<crate::helpers::DirEntry>;
    type IntoIter = crate::helpers::DirIter;

    fn into_iter(self) -> DirIter {
        DirIter {
            root: self.path,
            dir_iter: self.build.dir_options.into_iter(),
            filter: self.build.filter_options.glob,
        }
    }
}

#[cfg(feature = "ascii")]
fn list_ascii(path: impl AsRef<std::path::Path>) -> Result<crate::helpers::Tree, anyhow::Error> {
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
