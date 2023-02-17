mod copy;
mod delete;
mod list;
mod read;
mod resolve;
mod write;

use camino::{Utf8Path, Utf8PathBuf};
pub use copy::PathCopy;
pub use delete::PathDelete;
pub use list::PathList;
pub use read::PathRead;
pub use resolve::PathResolve;
pub use write::PathWrite;

use crate::FileManagerError;

pub trait PathBase {
    fn make_dirs(&self) -> crate::Result<Utf8PathBuf>;
}

impl PathBase for Utf8Path {
    fn make_dirs(&self) -> crate::Result<Utf8PathBuf> {
        std::fs::create_dir_all(self).map_err(|source| {
            FileManagerError::new(format!("could not create directory {self}"), source.into())
        })?;
        Ok(self.to_path_buf())
    }
}
