use super::{ctx_dent, ctx_depth, ctx_depth_path};
use super::{DirLister, WalkDirEntry};
use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8Path;
use camino::Utf8PathBuf;
use same_file::Handle;
use std::cmp::min;
use std::cmp::Ordering;
use std::fs;
use std::fs::ReadDir;
use std::{io, result, vec};

/// Like try, but for iterators that return [`Option<Result<_, _>>`].
///
/// [`Option<Result<_, _>>`]: https://doc.rust-lang.org/stable/std/option/enum.Option.html
macro_rules! itry {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(err) => return Some(Err(From::from(err))),
        }
    };
}

/// An iterator for recursively descending into a directory.
///
/// A value with this type must be constructed with the [`WalkDir`] type, which
/// uses a builder pattern to set options such as min/max depth, max open file
/// descriptors and whether the iterator should follow symbolic links. After
/// constructing a `WalkDir`, call [`.into_iter()`] at the end of the chain.
///
/// The order of elements yielded by this iterator is unspecified.
///
/// [`WalkDir`]: struct.WalkDir.html
/// [`.into_iter()`]: struct.WalkDir.html#into_iter.v
#[derive(Debug)]
pub struct DirIter {
    /// Options specified in the builder. Depths, max fds, etc.
    pub(crate) opts: DirLister,
    /// The start path.
    ///
    /// This is only `Some(...)` at the beginning. After the first iteration,
    /// this is always `None`.
    pub(crate) start: Option<Utf8PathBuf>,
    /// A stack of open (up to max fd) or closed handles to directories.
    /// An open handle is a plain [`fs::ReadDir`] while a closed handle is
    /// a `Vec<fs::DirEntry>` corresponding to the as-of-yet consumed entries.
    ///
    /// [`fs::ReadDir`]: https://doc.rust-lang.org/stable/std/fs/struct.ReadDir.html
    pub(crate) stack_list: Vec<DirList>,
    /// A stack of file paths.
    ///
    /// This is *only* used when [`follow_links`] is enabled. In all other
    /// cases this stack is empty.
    ///
    /// [`follow_links`]: struct.WalkDir.html#method.follow_links
    pub(crate) stack_path: Vec<Ancestor>,
    /// An index into `stack_list` that points to the oldest open directory
    /// handle. If the maximum fd limit is reached and a new directory needs to
    /// be read, the handle at this index is closed before the new directory is
    /// opened.
    pub(crate) oldest_opened: usize,
    /// The current depth of iteration (the length of the stack at the
    /// beginning of each iteration).
    pub(crate) depth: usize,
    /// A list of DirEntries corresponding to directories, that are
    /// yielded after their contents has been fully yielded. This is only
    /// used when `contents_first` is enabled.
    pub(crate) deferred_dirs: Vec<WalkDirEntry>,
    /// The device of the root file path when the first call to `next` was
    /// made.
    ///
    /// If the `same_file_system` option isn't enabled, then this is always
    /// `None`. Conversely, if it is enabled, this is always `Some(...)` after
    /// handling the root path.
    pub(crate) root_device: Option<u64>,
}

/// An ancestor is an item in the directory tree traversed by walkdir, and is
/// used to check for loops in the tree when traversing symlinks.
#[derive(Debug)]
pub(crate) struct Ancestor {
    /// The path of this ancestor.
    path: Utf8PathBuf,
    /// An open file to this ancesor. This is only used on Windows where
    /// opening a file handle appears to be quite expensive, so we choose to
    /// cache it. This comes at the cost of not respecting the file descriptor
    /// limit set by the user.
    #[cfg(windows)]
    handle: Handle,
}

impl Ancestor {
    /// Create a new ancestor from the given directory path.
    #[cfg(windows)]
    fn new(dent: &WalkDirEntry) -> io::Result<Ancestor> {
        let handle = Handle::from_path(dent.path())?;
        Ok(Ancestor {
            path: dent.path().to_path_buf(),
            handle: handle,
        })
    }

    /// Create a new ancestor from the given directory path.
    #[cfg(not(windows))]
    fn new(dent: &WalkDirEntry) -> io::Result<Ancestor> {
        Ok(Ancestor {
            path: dent.path().to_path_buf(),
        })
    }

    /// Returns true if and only if the given open file handle corresponds to
    /// the same directory as this ancestor.
    #[cfg(windows)]
    fn is_same(&self, child: &Handle) -> io::Result<bool> {
        Ok(child == &self.handle)
    }

    /// Returns true if and only if the given open file handle corresponds to
    /// the same directory as this ancestor.
    #[cfg(not(windows))]
    fn is_same(&self, child: &Handle) -> io::Result<bool> {
        Ok(child == &Handle::from_path(&self.path)?)
    }
}

/// A sequence of unconsumed directory entries.
///
/// This represents the opened or closed state of a directory handle. When
/// open, future entries are read by iterating over the raw `fs::ReadDir`.
/// When closed, all future entries are read into memory. Iteration then
/// proceeds over a [`Vec<fs::DirEntry>`].
///
/// [`fs::ReadDir`]: https://doc.rust-lang.org/stable/std/fs/struct.ReadDir.html
/// [`Vec<fs::DirEntry>`]: https://doc.rust-lang.org/stable/std/vec/struct.Vec.html
#[derive(Debug)]
pub(crate) enum DirList {
    /// An opened handle.
    ///
    /// This includes the depth of the handle itself.
    ///
    /// If there was an error with the initial [`fs::read_dir`] call, then it
    /// is stored here. (We use an [`Option<...>`] to make yielding the error
    /// exactly once simpler.)
    ///
    /// [`fs::read_dir`]: https://doc.rust-lang.org/stable/std/fs/fn.read_dir.html
    /// [`Option<...>`]: https://doc.rust-lang.org/stable/std/option/enum.Option.html
    Opened {
        depth: usize,
        it: result::Result<ReadDir, Option<anyhow::Error>>,
    },
    /// A closed handle.
    ///
    /// All remaining directory entries are read into memory.
    Closed(vec::IntoIter<Result<WalkDirEntry>>),
}

impl Iterator for DirIter {
    type Item = Result<WalkDirEntry>;
    /// Advances the iterator and returns the next value.
    ///
    /// # Errors
    ///
    /// If the iterator fails to retrieve the next value, this method returns
    /// an error value. The error will be wrapped in an Option::Some.
    fn next(&mut self) -> Option<Result<WalkDirEntry>> {
        if let Some(start) = self.start.take() {
            if self.opts.same_file_system {
                let result = super::device_num(&start).context(ctx_depth_path(0, &start));
                self.root_device = Some(itry!(result));
            }
            let dent = itry!(WalkDirEntry::from_path(0, start, false));
            if let Some(result) = self.handle_entry(dent) {
                return Some(result);
            }
        }
        while !self.stack_list.is_empty() {
            self.depth = self.stack_list.len();
            if let Some(dentry) = self.get_deferred_dir() {
                return Some(Ok(dentry));
            }
            if self.depth > self.opts.max_depth {
                // If we've exceeded the max depth, pop the current dir
                // so that we don't descend.
                self.pop();
                continue;
            }
            // Unwrap is safe here because we've verified above that
            // `self.stack_list` is not empty
            let next = self
                .stack_list
                .last_mut()
                .expect("BUG: stack should be non-empty")
                .next();
            match next {
                None => self.pop(),
                Some(Err(err)) => return Some(Err(err)),
                Some(Ok(dent)) => {
                    if let Some(result) = self.handle_entry(dent) {
                        return Some(result);
                    }
                }
            }
        }
        if self.opts.contents_first {
            self.depth = self.stack_list.len();
            if let Some(dentry) = self.get_deferred_dir() {
                return Some(Ok(dentry));
            }
        }
        None
    }
}

impl DirIter {
    pub fn skip_current_dir(&mut self) {
        if !self.stack_list.is_empty() {
            self.pop();
        }
    }

    pub fn filter_entry<P>(self, predicate: P) -> FilterEntry<Self, P>
    where
        P: FnMut(&WalkDirEntry) -> bool,
    {
        FilterEntry {
            it: self,
            predicate: predicate,
        }
    }

    fn handle_entry(&mut self, mut dent: WalkDirEntry) -> Option<Result<WalkDirEntry>> {
        if self.opts.follow_links && dent.file_type().is_symlink() {
            dent = itry!(self.follow(dent));
        }
        let is_normal_dir = !dent.file_type().is_symlink() && dent.is_dir();
        if is_normal_dir {
            if self.opts.same_file_system && dent.depth() > 0 {
                if itry!(self.is_same_file_system(&dent)) {
                    itry!(self.push(&dent));
                }
            } else {
                itry!(self.push(&dent));
            }
        } else if dent.depth() == 0 && dent.file_type().is_symlink() {
            // As a special case, if we are processing a root entry, then we
            // always follow it even if it's a symlink and follow_links is
            // false. We are careful to not let this change the semantics of
            // the DirEntry however. Namely, the DirEntry should still respect
            // the follow_links setting. When it's disabled, it should report
            // itself as a symlink. When it's enabled, it should always report
            // itself as the target.
            let md = itry!(fs::metadata(dent.path()).context(ctx_dent(&dent)));
            if md.file_type().is_dir() {
                itry!(self.push(&dent));
            }
        }
        if is_normal_dir && self.opts.contents_first {
            self.deferred_dirs.push(dent);
            None
        } else if self.skippable() {
            None
        } else {
            Some(Ok(dent))
        }
    }

    fn get_deferred_dir(&mut self) -> Option<WalkDirEntry> {
        if self.opts.contents_first {
            if self.depth < self.deferred_dirs.len() {
                // Unwrap is safe here because we've guaranteed that
                // `self.deferred_dirs.len()` can never be less than 1
                let deferred: WalkDirEntry = self
                    .deferred_dirs
                    .pop()
                    .expect("BUG: deferred_dirs should be non-empty");
                if !self.skippable() {
                    return Some(deferred);
                }
            }
        }
        None
    }

    fn push(&mut self, dent: &WalkDirEntry) -> Result<()> {
        // Make room for another open file descriptor if we've hit the max.
        let free = self
            .stack_list
            .len()
            .checked_sub(self.oldest_opened)
            .unwrap();
        if free == self.opts.max_open {
            self.stack_list[self.oldest_opened].close();
        }
        // Open a handle to reading the directory's entries.
        let rd = fs::read_dir(dent.path())
            .map_err(|err| Some(anyhow!(err).context(ctx_depth_path(self.depth, dent.path()))));
        let mut list = DirList::Opened {
            depth: self.depth,
            it: rd,
        };
        if let Some(ref mut cmp) = self.opts.sorter {
            let mut entries: Vec<_> = list.collect();
            entries.sort_by(|a, b| match (a, b) {
                (&Ok(ref a), &Ok(ref b)) => cmp(a, b),
                (&Err(_), &Err(_)) => Ordering::Equal,
                (&Ok(_), &Err(_)) => Ordering::Greater,
                (&Err(_), &Ok(_)) => Ordering::Less,
            });
            list = DirList::Closed(entries.into_iter());
        }
        if self.opts.follow_links {
            let ancestor = Ancestor::new(&dent).context(ctx_depth(self.depth))?;
            self.stack_path.push(ancestor);
        }
        // We push this after stack_path since creating the Ancestor can fail.
        // If it fails, then we return the error and won't descend.
        self.stack_list.push(list);
        // If we had to close out a previous directory stream, then we need to
        // increment our index the oldest still-open stream. We do this only
        // after adding to our stack, in order to ensure that the oldest_opened
        // index remains valid. The worst that can happen is that an already
        // closed stream will be closed again, which is a no-op.
        //
        // We could move the close of the stream above into this if-body, but
        // then we would have more than the maximum number of file descriptors
        // open at a particular point in time.
        if free == self.opts.max_open {
            // Unwrap is safe here because self.oldest_opened is guaranteed to
            // never be greater than `self.stack_list.len()`, which implies
            // that the subtraction won't underflow and that adding 1 will
            // never overflow.
            self.oldest_opened = self.oldest_opened.checked_add(1).unwrap();
        }
        Ok(())
    }

    fn pop(&mut self) {
        self.stack_list
            .pop()
            .expect("BUG: cannot pop from empty stack");
        if self.opts.follow_links {
            self.stack_path
                .pop()
                .expect("BUG: list/path stacks out of sync");
        }
        // If everything in the stack is already closed, then there is
        // room for at least one more open descriptor and it will
        // always be at the top of the stack.
        self.oldest_opened = min(self.oldest_opened, self.stack_list.len());
    }

    fn follow(&self, mut dent: WalkDirEntry) -> Result<WalkDirEntry> {
        dent = WalkDirEntry::from_path(self.depth, dent.path().to_path_buf(), true)?;
        // The only way a symlink can cause a loop is if it points
        // to a directory. Otherwise, it always points to a leaf
        // and we can omit any loop checks.
        if dent.is_dir() {
            self.check_loop(dent.path())?;
        }
        Ok(dent)
    }

    fn check_loop<P: AsRef<Utf8Path>>(&self, child: P) -> Result<()> {
        let hchild =
            Handle::from_path(&child.as_ref().as_std_path()).context(ctx_depth(self.depth))?;
        for ancestor in self.stack_path.iter().rev() {
            let is_same = ancestor.is_same(&hchild).context(ctx_depth(self.depth))?;
            if is_same {
                bail!(
                    "File system loop found: {} points to an ancestor {}",
                    child.as_ref(),
                    &ancestor.path,
                );
            }
        }
        Ok(())
    }

    fn is_same_file_system(&mut self, dent: &WalkDirEntry) -> Result<bool> {
        let dent_device = super::device_num(dent.path()).context(ctx_dent(&dent))?;
        Ok(self
            .root_device
            .map(|d| d == dent_device)
            .expect("BUG: called is_same_file_system without root device"))
    }

    fn skippable(&self) -> bool {
        self.depth < self.opts.min_depth || self.depth > self.opts.max_depth
    }
}

impl DirList {
    fn close(&mut self) {
        if let DirList::Opened { .. } = *self {
            *self = DirList::Closed(self.collect::<Vec<_>>().into_iter());
        }
    }
}

impl Iterator for DirList {
    type Item = Result<WalkDirEntry>;

    #[inline(always)]
    fn next(&mut self) -> Option<Result<WalkDirEntry>> {
        match *self {
            DirList::Closed(ref mut it) => it.next(),
            DirList::Opened { depth, ref mut it } => match *it {
                Err(ref mut err) => err.take().map(Err),
                Ok(ref mut rd) => rd.next().map(|r| match r {
                    Ok(r) => WalkDirEntry::from_entry(depth + 1, &r),
                    Err(err) => Err(err).context(ctx_depth(depth + 1)),
                }),
            },
        }
    }
}

/// A recursive directory iterator that skips entries.
///
/// Values of this type are created by calling [`.filter_entry()`] on an
/// `IntoIter`, which is formed by calling [`.into_iter()`] on a `WalkDir`.
///
/// Directories that fail the predicate `P` are skipped. Namely, they are
/// never yielded and never descended into.
///
/// Entries that are skipped with the [`min_depth`] and [`max_depth`] options
/// are not passed through this filter.
///
/// If opening a handle to a directory resulted in an error, then it is yielded
/// and no corresponding call to the predicate is made.
///
/// Type parameter `I` refers to the underlying iterator and `P` refers to the
/// predicate, which is usually `FnMut(&DirEntry) -> bool`.
///
/// [`.filter_entry()`]: struct.IntoIter.html#method.filter_entry
/// [`.into_iter()`]: struct.WalkDir.html#into_iter.v
/// [`min_depth`]: struct.WalkDir.html#method.min_depth
/// [`max_depth`]: struct.WalkDir.html#method.max_depth
#[derive(Debug)]
pub struct FilterEntry<I, P> {
    it: I,
    predicate: P,
}

impl<P> Iterator for FilterEntry<DirIter, P>
where
    P: FnMut(&WalkDirEntry) -> bool,
{
    type Item = Result<WalkDirEntry>;

    /// Advances the iterator and returns the next value.
    ///
    /// # Errors
    ///
    /// If the iterator fails to retrieve the next value, this method returns
    /// an error value. The error will be wrapped in an `Option::Some`.
    fn next(&mut self) -> Option<Result<WalkDirEntry>> {
        loop {
            let dent = match self.it.next() {
                None => return None,
                Some(result) => itry!(result),
            };
            if !(self.predicate)(&dent) {
                if dent.is_dir() {
                    self.it.skip_current_dir();
                }
                continue;
            }
            return Some(Ok(dent));
        }
    }
}

impl<P> FilterEntry<DirIter, P>
where
    P: FnMut(&WalkDirEntry) -> bool,
{
    pub fn filter_entry(self, predicate: P) -> FilterEntry<Self, P> {
        FilterEntry {
            it: self,
            predicate: predicate,
        }
    }

    pub fn skip_current_dir(&mut self) {
        self.it.skip_current_dir();
    }
}
