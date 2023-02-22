use crate::FileManagerError;
use anyhow::anyhow;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs::File;

#[derive(Clone)]
pub enum ExistingFile {
    Overwrite,
    Skip,
    Error,
}

#[derive(Clone)]
pub enum MissingFile {
    Create,
    Skip,
    Error,
}

pub enum FileOption {
    Existing(ExistingFile),
    Missing(MissingFile),
}

impl From<MissingFile> for FileOption {
    fn from(missing: MissingFile) -> Self {
        Self::Missing(missing)
    }
}
impl From<ExistingFile> for FileOption {
    fn from(existing: ExistingFile) -> Self {
        Self::Existing(existing)
    }
}

#[derive(Clone)]
pub struct DestinationFileOptions {
    pub existing_file: ExistingFile,
    pub missing_file: MissingFile,
}

impl Default for DestinationFileOptions {
    fn default() -> Self {
        Self {
            existing_file: ExistingFile::Overwrite,
            missing_file: MissingFile::Create,
        }
    }
}

impl DestinationFileOptions {
    pub fn set(&mut self, file: impl Into<FileOption>) {
        match file.into() {
            FileOption::Existing(existing) => self.existing_file = existing,
            FileOption::Missing(missing) => self.missing_file = missing,
        }
    }

    pub fn get_path(&self, dest: &Utf8Path) -> anyhow::Result<Option<Utf8PathBuf>> {
        if dest.exists() {
            match self.existing_file {
                ExistingFile::Overwrite => Ok(Some(dest.to_path_buf())),
                ExistingFile::Skip => Ok(None),
                ExistingFile::Error => Err(anyhow!("file already exists: {dest}")),
            }
        } else {
            match self.missing_file {
                MissingFile::Create => Ok(Some(dest.to_path_buf())),
                MissingFile::Skip => Ok(None),
                MissingFile::Error => Err(anyhow!("missing file: {dest}")),
            }
        }
    }

    pub fn get_file(&self, dest: &Utf8Path) -> anyhow::Result<Option<File>> {
        if dest.exists() {
            match self.existing_file {
                ExistingFile::Overwrite => Ok(Some(File::create(dest).map_err(|source| {
                    FileManagerError::new(
                        format!("could not overwrite file: {dest}"),
                        source.into(),
                    )
                })?)),
                ExistingFile::Skip => Ok(None),
                ExistingFile::Error => Err(anyhow!("file already exists: {dest}")),
            }
        } else {
            match self.missing_file {
                MissingFile::Create => Ok(Some(File::create(dest).map_err(|source| {
                    FileManagerError::new(format!("could not create file: {dest}"), source.into())
                })?)),
                MissingFile::Skip => Ok(None),
                MissingFile::Error => Err(anyhow!("missing file: {dest}")),
            }
        }
    }
}
