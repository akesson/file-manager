use camino::{Utf8Path, Utf8PathBuf};

pub trait PathDelete {
    fn delete(&self) -> DeleteBuilder;
}

impl PathDelete for Utf8Path {
    fn delete(&self) -> DeleteBuilder {
        DeleteBuilder::new(self.to_path_buf())
    }
}

pub struct DeleteBuilder {
    path: Utf8PathBuf,
}

impl DeleteBuilder {
    fn new(path: Utf8PathBuf) -> Self {
        Self { path }
    }

    pub fn file(self) -> crate::Result<Utf8PathBuf> {
        std::fs::remove_file(&self.path).map_err(|source| {
            crate::FileManagerError::new(
                format!("could not delete file {}", self.path),
                source.into(),
            )
        })?;
        Ok(self.path.to_path_buf())
    }

    pub fn dir(self) -> DeleteDir {
        DeleteDir::new(self)
    }
}

pub struct DeleteDir {
    path: Utf8PathBuf,
}

impl DeleteDir {
    fn new(builder: DeleteBuilder) -> Self {
        Self { path: builder.path }
    }

    pub fn all(self) -> crate::Result<Utf8PathBuf> {
        std::fs::remove_dir_all(&self.path).map_err(|source| {
            crate::FileManagerError::new(
                format!("could not delete directory {}", self.path),
                source.into(),
            )
        })?;
        Ok(self.path.to_path_buf())
    }

    pub fn empty(self) -> crate::Result<Utf8PathBuf> {
        std::fs::remove_dir(&self.path).map_err(|source| {
            crate::FileManagerError::new(
                format!("could not delete directory {}", self.path),
                source.into(),
            )
        })?;
        Ok(self.path.to_path_buf())
    }
}
