use std::cmp::Ordering;
use std::fmt;
use std::result;
use std::vec;

use camino::Utf8Path;
use camino::Utf8PathBuf;

pub use super::DirEntry;
#[cfg(unix)]
pub use super::DirEntryExt;

pub struct DirLister {
    pub(crate) root: Utf8PathBuf,
    pub(crate) follow_links: bool,
    pub(crate) max_open: usize,
    pub(crate) min_depth: usize,
    pub(crate) max_depth: usize,
    pub(crate) sorter:
        Option<Box<dyn FnMut(&DirEntry, &DirEntry) -> Ordering + Send + Sync + 'static>>,
    pub(crate) contents_first: bool,
    pub(crate) same_file_system: bool,
}

impl fmt::Debug for DirLister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        let sorter_str = if self.sorter.is_some() {
            // FnMut isn't `Debug`
            "Some(...)"
        } else {
            "None"
        };
        f.debug_struct("WalkDirOptions")
            .field("follow_links", &self.follow_links)
            .field("max_open", &self.max_open)
            .field("min_depth", &self.min_depth)
            .field("max_depth", &self.max_depth)
            .field("sorter", &sorter_str)
            .field("contents_first", &self.contents_first)
            .field("same_file_system", &self.same_file_system)
            .finish()
    }
}

impl DirLister {
    /// Create a builder for a recursive directory iterator starting at the
    /// file path `root`. If `root` is a directory, then it is the first item
    /// yielded by the iterator. If `root` is a file, then it is the first
    /// and only item yielded by the iterator. If `root` is a symlink, then it
    /// is always followed for the purposes of directory traversal. (A root
    /// `DirEntry` still obeys its documentation with respect to symlinks and
    /// the `follow_links` setting.)
    pub fn new<P: AsRef<Utf8Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            follow_links: false,
            max_open: 10,
            min_depth: 0,
            max_depth: ::std::usize::MAX,
            sorter: None,
            contents_first: false,
            same_file_system: false,
        }
    }

    /// Set the minimum depth of entries yielded by the iterator.
    ///
    /// The smallest depth is `0` and always corresponds to the path given
    /// to the `new` function on this type. Its direct descendents have depth
    /// `1`, and their descendents have depth `2`, and so on.
    pub fn min_depth(&mut self, depth: usize) {
        self.min_depth = depth;
        if self.min_depth > self.max_depth {
            self.min_depth = self.max_depth;
        }
    }

    /// Set the maximum depth of entries yield by the iterator.
    ///
    /// The smallest depth is `0` and always corresponds to the path given
    /// to the `new` function on this type. Its direct descendents have depth
    /// `1`, and their descendents have depth `2`, and so on.
    ///
    /// Note that this will not simply filter the entries of the iterator, but
    /// it will actually avoid descending into directories when the depth is
    /// exceeded.
    pub fn max_depth(&mut self, depth: usize) {
        self.max_depth = depth;
        if self.max_depth < self.min_depth {
            self.max_depth = self.min_depth;
        }
    }

    /// Follow symbolic links. By default, this is disabled.
    ///
    /// When `yes` is `true`, symbolic links are followed as if they were
    /// normal directories and files. If a symbolic link is broken or is
    /// involved in a loop, an error is yielded.
    ///
    /// When enabled, the yielded [`DirEntry`] values represent the target of
    /// the link while the path corresponds to the link. See the [`DirEntry`]
    /// type for more details.
    ///
    /// [`DirEntry`]: struct.DirEntry.html
    pub fn follow_links(&mut self, yes: bool) {
        self.follow_links = yes;
    }

    /// Set the maximum number of simultaneously open file descriptors used
    /// by the iterator.
    ///
    /// `n` must be greater than or equal to `1`. If `n` is `0`, then it is set
    /// to `1` automatically. If this is not set, then it defaults to some
    /// reasonably low number.
    ///
    /// This setting has no impact on the results yielded by the iterator
    /// (even when `n` is `1`). Instead, this setting represents a trade off
    /// between scarce resources (file descriptors) and memory. Namely, when
    /// the maximum number of file descriptors is reached and a new directory
    /// needs to be opened to continue iteration, then a previous directory
    /// handle is closed and has its unyielded entries stored in memory. In
    /// practice, this is a satisfying trade off because it scales with respect
    /// to the *depth* of your file tree. Therefore, low values (even `1`) are
    /// acceptable.
    ///
    /// Note that this value does not impact the number of system calls made by
    /// an exhausted iterator.
    ///
    /// # Platform behavior
    ///
    /// On Windows, if `follow_links` is enabled, then this limit is not
    /// respected. In particular, the maximum number of file descriptors opened
    /// is proportional to the depth of the directory tree traversed.
    pub fn max_open(&mut self, mut n: usize) {
        if n == 0 {
            n = 1;
        }
        self.max_open = n;
    }

    pub fn sort_by<F>(&mut self, cmp: F)
    where
        F: FnMut(&DirEntry, &DirEntry) -> Ordering + Send + Sync + 'static,
    {
        self.sorter = Some(Box::new(cmp));
    }

    pub fn sort_by_key<K, F>(&mut self, mut cmp: F)
    where
        F: FnMut(&DirEntry) -> K + Send + Sync + 'static,
        K: Ord,
    {
        self.sort_by(move |a, b| cmp(a).cmp(&cmp(b)))
    }

    pub fn sort_by_file_name(&mut self) {
        self.sort_by(|a, b| a.file_name().cmp(b.file_name()))
    }

    pub fn contents_first(&mut self, yes: bool) {
        self.contents_first = yes;
    }

    /// Do not cross file system boundaries.
    ///
    /// When this option is enabled, directory traversal will not descend into
    /// directories that are on a different file system from the root path.
    ///
    /// Currently, this option is only supported on Unix and Windows. If this
    /// option is used on an unsupported platform, then directory traversal
    /// will immediately return an error and will not yield any entries.
    pub fn same_file_system(&mut self, yes: bool) {
        self.same_file_system = yes;
    }
}

impl IntoIterator for DirLister {
    type Item = anyhow::Result<DirEntry>;
    type IntoIter = super::DirIter;

    fn into_iter(self) -> super::DirIter {
        let start = self.root.clone();
        super::DirIter {
            opts: self,
            start: Some(start),
            stack_list: vec![],
            stack_path: vec![],
            oldest_opened: 0,
            depth: 0,
            deferred_dirs: vec![],
            root_device: None,
        }
    }
}
