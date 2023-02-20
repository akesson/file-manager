use std::ops::DerefMut;

use camino::Utf8PathBuf;

use super::{DirEntry, DirIter};

struct WalkDirOptions {
    follow_links: bool,
    min_depth: usize,
    max_depth: usize,
    same_file_system: bool,
}

impl Default for WalkDirOptions {
    fn default() -> Self {
        WalkDirOptions {
            follow_links: false,
            // max_open: 10,
            min_depth: 0,
            max_depth: ::std::usize::MAX,
            // sorter: None,
            // contents_first: false,
            same_file_system: false,
        }
    }
}

pub struct ListerOptions {
    pub(crate) path: Utf8PathBuf,
    #[cfg(feature = "glob")]
    glob: super::GlobBuilder,
    options: WalkDirOptions,
}

pub trait Lister
where
    Self: Sized + DerefMut<Target = ListerOptions>,
{
    fn min_depth(mut self, depth: usize) -> Self {
        self.options.min_depth = depth;
        self
    }

    fn max_depth(mut self, depth: usize) -> Self {
        self.options.max_depth = depth;
        self
    }

    fn follow_links(mut self) -> Self {
        self.options.follow_links = true;
        self
    }

    fn same_file_system(mut self) -> Self {
        self.options.same_file_system = true;
        self
    }

    #[cfg(feature = "glob")]
    fn exclude(mut self, glob: impl AsRef<str>) -> crate::Result<Self> {
        self.glob.exclude(glob)?;
        Ok(self)
    }

    #[cfg(feature = "glob")]
    fn include(mut self, glob: impl AsRef<str>) -> crate::Result<Self> {
        self.glob.include(glob)?;
        Ok(self)
    }
}

impl ListerOptions {
    pub fn new(path: Utf8PathBuf) -> Self {
        Self {
            path,
            #[cfg(feature = "glob")]
            glob: Default::default(),
            options: Default::default(),
        }
    }
}

impl IntoIterator for ListerOptions {
    type Item = crate::Result<DirEntry>;
    type IntoIter = DirIter;

    fn into_iter(self) -> Self::IntoIter {
        let dir_iter = walkdir::WalkDir::new(&self.path)
            .max_depth(self.options.max_depth)
            .min_depth(self.options.min_depth)
            .follow_links(self.options.follow_links)
            .same_file_system(self.options.same_file_system)
            .into_iter();

        DirIter {
            root: self.path,
            dir_iter,
            #[cfg(feature = "glob")]
            filter: self.glob.build(),
        }
    }
}
