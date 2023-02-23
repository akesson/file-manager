use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

use crate::FileManagerError;
use dir_iter::{WalkDirEntry, WalkDirIter};

pub struct DirIter {
    pub(crate) root: Utf8PathBuf,
    pub(crate) dir_iter: WalkDirIter,
    #[cfg(feature = "glob")]
    pub(crate) filter: super::GlobFilter,
}

pub struct DirEntry {
    pub(crate) entry: WalkDirEntry,
    pub(crate) path: Utf8PathBuf,
    pub(crate) relative: Utf8PathBuf,
}

impl DirEntry {
    fn new(entry: WalkDirEntry, root: &Utf8Path) -> anyhow::Result<Self> {
        let path = entry.path().to_path_buf();
        let relative = path
            .strip_prefix(&root)
            .context(format!(
                "could not remove the root path {root} from {path} please create a bug report"
            ))?
            .to_path_buf();

        Ok(Self {
            entry,
            path,
            relative,
        })
    }

    pub fn path(&self) -> &Utf8Path {
        &self.path
    }

    pub fn relative_path(&self) -> &Utf8Path {
        &self.relative
    }

    pub fn file_name(&self) -> &str {
        self.path.file_name().unwrap()
    }
}

impl std::ops::Deref for DirEntry {
    type Target = WalkDirEntry;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl Iterator for DirIter {
    // using crate::Result as the iterator is user facing
    type Item = crate::Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.dir_iter.next() {
            match next.map(|p| DirEntry::new(p, &self.root)) {
                Ok(Ok(entry)) => {
                    if self.filter.is_match(&entry.relative_path().as_str()) {
                        return Some(Ok(entry));
                    } else {
                        // println!("next: {:?}", entry.relative_path());
                    }
                }
                Ok(Err(source)) => {
                    return Some(Err(FileManagerError::new(
                        format!("error while listing contents of {}", self.root),
                        source,
                    )))
                }
                Err(source) => {
                    return Some(Err(FileManagerError::new(
                        format!("error while listing contents of {}", self.root),
                        source.into(),
                    )))
                }
            }
        }
        None
    }
}
