use camino::{Utf8Path, Utf8PathBuf};
use std::{cmp::Ordering, path::Path};

use crate::helpers::{DirEntry, DirIter};

pub trait PathList {
    fn list(&self) -> ListBuilder;
}

impl PathList for Utf8Path {
    fn list(&self) -> ListBuilder {
        ListBuilder::new(self.to_path_buf())
    }
}

pub struct ListBuilder {
    path: Utf8PathBuf,
    #[cfg(feature = "glob")]
    glob: crate::helpers::GlobBuilder,
    walkdir: walkdir::WalkDir,
}

impl ListBuilder {
    fn new(path: Utf8PathBuf) -> Self {
        let walkdir = walkdir::WalkDir::new(&path);
        Self {
            path,
            #[cfg(feature = "glob")]
            glob: Default::default(),
            walkdir,
        }
    }

    pub fn min_depth(mut self, depth: usize) -> Self {
        self.walkdir = self.walkdir.min_depth(depth);
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.walkdir = self.walkdir.max_depth(depth);
        self
    }

    pub fn follow_links(mut self) -> Self {
        self.walkdir = self.walkdir.follow_links(true);
        self
    }

    pub fn sort_by<F>(mut self, cmp: F) -> Self
    where
        F: FnMut(&walkdir::DirEntry, &walkdir::DirEntry) -> Ordering + Send + Sync + 'static,
    {
        self.walkdir = self.walkdir.sort_by(cmp);
        self
    }

    pub fn sort_by_file_name(mut self) -> Self {
        self.walkdir = self.walkdir.sort_by_file_name();
        self
    }

    pub fn sort_by_key<K, F>(mut self, f: F) -> Self
    where
        F: FnMut(&walkdir::DirEntry) -> K + Send + Sync + 'static,
        K: Ord,
    {
        self.walkdir = self.walkdir.sort_by_key(f);
        self
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

    #[cfg(feature = "glob")]
    pub fn exclude(mut self, glob: impl AsRef<str>) -> crate::Result<Self> {
        self.glob.exclude(glob)?;
        Ok(self)
    }

    #[cfg(feature = "glob")]
    pub fn include(mut self, glob: impl AsRef<str>) -> crate::Result<Self> {
        self.glob.include(glob)?;
        Ok(self)
    }

    #[cfg(feature = "ascii")]
    pub fn ascii(self) -> crate::Result<String> {
        let tree = list_ascii(&self.path).map_err(|source| {
            crate::FileManagerError::new(
                format!("Could not create ascii listing of {:?}", self.path),
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
        DirIter {
            root: self.path,
            dir_iter: self.walkdir.into_iter(),
            #[cfg(feature = "glob")]
            filter: self.glob.build(),
        }
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
