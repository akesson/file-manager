#[cfg(test)]
mod tests;

mod dir_entry;
mod dir_iter;
mod dir_lister;

pub use crate::dir_entry::{DirEntryExt, WalkDirEntry};
pub use crate::dir_iter::{DirIter, FilterEntry};
pub use crate::dir_lister::DirLister;
use camino::Utf8Path;

use std::io;
use std::path::Path;

#[cfg(unix)]
pub fn device_num<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    use std::os::unix::fs::MetadataExt;

    path.as_ref().metadata().map(|md| md.dev())
}

#[cfg(windows)]
pub fn device_num<P: AsRef<Path>>(path: P) -> io::Result<u64> {
    use winapi_util::{file, Handle};

    let h = Handle::from_path_any(path)?;
    file::information(h).map(|info| info.volume_serial_number())
}

#[cfg(not(any(unix, windows)))]
pub fn device_num<P: AsRef<Path>>(_: P) -> io::Result<u64> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "walkdir: same_file_system option not supported on this platform",
    ))
}

fn ctx_depth(depth: usize) -> String {
    format!("error at depth {depth}")
}
fn ctx_depth_path(depth: usize, path: impl AsRef<Utf8Path>) -> String {
    format!("error at depth {} for: {}", depth, path.as_ref())
}

fn ctx_dent(dent: &WalkDirEntry) -> String {
    format!("error at depth {} for: {}", dent.depth(), dent.path())
}
