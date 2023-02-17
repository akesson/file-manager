use crate::Result;
use camino::{Utf8Path, Utf8PathBuf};
use std::path::Path;

pub trait PathCopy {
    /// Create a CopyBuilder for this path.
    ///
    /// Returns the destination path.
    fn copy(&self) -> CopyBuilder;
}

impl PathCopy for Utf8Path {
    fn copy(&self) -> CopyBuilder {
        CopyBuilder::new(self.to_path_buf())
    }
}

pub struct CopyBuilder {
    from: Utf8PathBuf,
}

impl CopyBuilder {
    pub fn new(from: Utf8PathBuf) -> Self {
        Self { from }
    }

    pub fn file(self) -> CopyFile {
        CopyFile::new(self)
    }

    pub fn dir(self) -> CopyDir {
        CopyDir::new(self)
    }

    pub fn to(self, to: impl AsRef<Path>) -> Result<Utf8PathBuf> {
        Ok(self.from)
    }
}
pub struct CopyFile {
    from: Utf8PathBuf,
    overwrite: bool,
}

impl CopyFile {
    pub fn new(builder: CopyBuilder) -> Self {
        Self {
            from: builder.from,
            overwrite: false,
        }
    }
}
pub struct CopyDir {
    from: Utf8PathBuf,
    overwrite: bool,
    contents: bool,
    exclude: Vec<String>,
    include: Vec<String>,
}

impl CopyDir {
    pub fn new(builder: CopyBuilder) -> Self {
        Self {
            from: builder.from,
            overwrite: false,
            contents: true,
            exclude: vec![],
            include: vec![],
        }
    }
}
