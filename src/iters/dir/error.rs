use std::error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use super::WalkDirEntry;

#[derive(Debug)]
pub struct DirError {
    depth: usize,
    inner: ErrorInner,
}

#[derive(Debug)]
enum ErrorInner {
    Io {
        path: Option<PathBuf>,
        err: io::Error,
    },
    Loop {
        ancestor: PathBuf,
        child: PathBuf,
    },
}

impl DirError {
    /// Returns the path associated with this error if one exists.
    ///
    /// For example, if an error occurred while opening a directory handle,
    /// the error will include the path passed to [`std::fs::read_dir`].
    ///
    /// [`std::fs::read_dir`]: https://doc.rust-lang.org/stable/std/fs/fn.read_dir.html
    pub fn path(&self) -> Option<&Path> {
        match self.inner {
            ErrorInner::Io { path: None, .. } => None,
            ErrorInner::Io {
                path: Some(ref path),
                ..
            } => Some(path),
            ErrorInner::Loop { ref child, .. } => Some(child),
        }
    }

    /// Returns the path at which a cycle was detected.
    ///
    /// If no cycle was detected, [`None`] is returned.
    ///
    /// A cycle is detected when a directory entry is equivalent to one of
    /// its ancestors.
    ///
    /// To get the path to the child directory entry in the cycle, use the
    /// [`path`] method.
    ///
    /// [`None`]: https://doc.rust-lang.org/stable/std/option/enum.Option.html#variant.None
    /// [`path`]: struct.Error.html#path
    pub fn loop_ancestor(&self) -> Option<&Path> {
        match self.inner {
            ErrorInner::Loop { ref ancestor, .. } => Some(ancestor),
            _ => None,
        }
    }

    /// Returns the depth at which this error occurred relative to the root.
    ///
    /// The smallest depth is `0` and always corresponds to the path given to
    /// the [`new`] function on [`WalkDir`]. Its direct descendents have depth
    /// `1`, and their descendents have depth `2`, and so on.
    ///
    /// [`new`]: struct.WalkDir.html#method.new
    /// [`WalkDir`]: struct.WalkDir.html
    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn io_error(&self) -> Option<&io::Error> {
        match self.inner {
            ErrorInner::Io { ref err, .. } => Some(err),
            ErrorInner::Loop { .. } => None,
        }
    }

    /// Similar to [`io_error`] except consumes self to convert to the original
    /// [`io::Error`] if one exists.
    ///
    /// [`io_error`]: struct.Error.html#method.io_error
    /// [`io::Error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html
    pub fn into_io_error(self) -> Option<io::Error> {
        match self.inner {
            ErrorInner::Io { err, .. } => Some(err),
            ErrorInner::Loop { .. } => None,
        }
    }

    pub(crate) fn from_path(depth: usize, pb: PathBuf, err: io::Error) -> Self {
        DirError {
            depth: depth,
            inner: ErrorInner::Io {
                path: Some(pb),
                err: err,
            },
        }
    }

    pub(crate) fn from_entry(dent: &WalkDirEntry, err: io::Error) -> Self {
        DirError {
            depth: dent.depth(),
            inner: ErrorInner::Io {
                path: Some(dent.path().to_path_buf()),
                err: err,
            },
        }
    }

    pub(crate) fn from_io(depth: usize, err: io::Error) -> Self {
        DirError {
            depth: depth,
            inner: ErrorInner::Io {
                path: None,
                err: err,
            },
        }
    }

    pub(crate) fn from_loop(depth: usize, ancestor: &Path, child: &Path) -> Self {
        DirError {
            depth: depth,
            inner: ErrorInner::Loop {
                ancestor: ancestor.to_path_buf(),
                child: child.to_path_buf(),
            },
        }
    }
}

impl error::Error for DirError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self.inner {
            ErrorInner::Io { ref err, .. } => err.description(),
            ErrorInner::Loop { .. } => "file system loop found",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.inner {
            ErrorInner::Io { ref err, .. } => Some(err),
            ErrorInner::Loop { .. } => None,
        }
    }
}

impl fmt::Display for DirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            ErrorInner::Io {
                path: None,
                ref err,
            } => err.fmt(f),
            ErrorInner::Io {
                path: Some(ref path),
                ref err,
            } => write!(f, "IO error for operation on {}: {}", path.display(), err),
            ErrorInner::Loop {
                ref ancestor,
                ref child,
            } => write!(
                f,
                "File system loop found: \
                 {} points to an ancestor {}",
                child.display(),
                ancestor.display()
            ),
        }
    }
}

impl From<DirError> for io::Error {
    /// Convert the [`Error`] to an [`io::Error`], preserving the original
    /// [`Error`] as the ["inner error"]. Note that this also makes the display
    /// of the error include the context.
    ///
    /// This is different from [`into_io_error`] which returns the original
    /// [`io::Error`].
    ///
    /// [`Error`]: struct.Error.html
    /// [`io::Error`]: https://doc.rust-lang.org/stable/std/io/struct.Error.html
    /// ["inner error"]: https://doc.rust-lang.org/std/io/struct.Error.html#method.into_inner
    /// [`into_io_error`]: struct.WalkDir.html#method.into_io_error
    fn from(walk_err: DirError) -> io::Error {
        let kind = match walk_err {
            DirError {
                inner: ErrorInner::Io { ref err, .. },
                ..
            } => err.kind(),
            DirError {
                inner: ErrorInner::Loop { .. },
                ..
            } => io::ErrorKind::Other,
        };
        io::Error::new(kind, walk_err)
    }
}
