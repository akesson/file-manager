use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

use crate::FileManagerError;

use super::Utf8PathExt;

pub struct TempDir {
    // keep a handle, because the tempdir is deleted when the handle is dropped
    _dir: tempfile::TempDir,
    root: Utf8PathBuf,
}

impl TempDir {
    pub fn new() -> crate::Result<Self> {
        Self::_new().map_err(|source| FileManagerError::new("could not create temp dir", source))
    }

    fn _new() -> anyhow::Result<Self> {
        let dir = tempfile::tempdir().context("Could not create temp dir")?;
        let root = Utf8PathBuf::from_path(dir.path())?;
        Ok(Self { _dir: dir, root })
    }

    pub fn path(&self) -> &Utf8Path {
        self.root.as_ref()
    }

    pub fn join(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
        self.path().join(path.as_ref())
    }
}
