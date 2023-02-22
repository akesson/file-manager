use camino::{Utf8Path, Utf8PathBuf};

use crate::helpers::DirIter;

use super::{Build, FilterOptions};

pub struct DirOptions {
    path: Utf8PathBuf,
    follow_links: bool,
    min_depth: usize,
    max_depth: usize,
    same_file_system: bool,
}

impl DirOptions {
    pub fn new(path: &Utf8Path) -> Self {
        DirOptions {
            path: path.to_path_buf(),
            follow_links: false,
            min_depth: 0,
            max_depth: ::std::usize::MAX,
            same_file_system: false,
        }
    }

    pub fn to_walkdir_iter(&self, path: &Utf8Path) -> walkdir::IntoIter {
        walkdir::WalkDir::new(path)
            .max_depth(self.max_depth)
            .min_depth(self.min_depth)
            .follow_links(self.follow_links)
            .same_file_system(self.same_file_system)
            .into_iter()
    }

    fn walkdir_iter(&self) -> walkdir::IntoIter {
        walkdir::WalkDir::new(&self.path)
            .max_depth(self.max_depth)
            .min_depth(self.min_depth)
            .follow_links(self.follow_links)
            .same_file_system(self.same_file_system)
            .into_iter()
    }

    pub fn filtered_iter(&self, filters: FilterOptions) -> DirIter {
        DirIter {
            root: self.path.clone(),
            dir_iter: self.walkdir_iter(),
            filter: filters.glob,
        }
    }
}

impl IntoIterator for DirOptions {
    type Item = Result<walkdir::DirEntry, walkdir::Error>;
    type IntoIter = walkdir::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.walkdir_iter()
    }
}

pub trait DirOptionsMut {
    fn dir_options(&mut self) -> &mut DirOptions;
}

impl<T> Build<T>
where
    T: DirOptionsMut,
{
    pub fn min_depth(mut self, depth: usize) -> Self {
        self.build.dir_options().min_depth = depth;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.build.dir_options().max_depth = depth;
        self
    }

    pub fn follow_links(mut self) -> Self {
        self.build.dir_options().follow_links = true;
        self
    }

    pub fn same_file_system(mut self) -> Self {
        self.build.dir_options().same_file_system = true;
        self
    }
}
