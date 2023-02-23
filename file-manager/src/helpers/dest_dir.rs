use super::Utf8PathExt;
use crate::FileManagerError;
use anyhow::anyhow;
use camino::Utf8Path;

#[derive(Clone)]
pub enum MissingDir {
    Create,
    Skip,
    Error,
}

#[derive(Clone)]
pub enum ExistingDir {
    Replace,
    Continue,
    Skip,
    Error,
}

pub enum DirOption {
    Existing(ExistingDir),
    Missing(MissingDir),
}

impl From<MissingDir> for DirOption {
    fn from(missing: MissingDir) -> Self {
        Self::Missing(missing)
    }
}

impl From<ExistingDir> for DirOption {
    fn from(existing: ExistingDir) -> Self {
        Self::Existing(existing)
    }
}
#[derive(Clone)]
pub struct DestinationDirOptions {
    pub existing_dir: ExistingDir,
    pub missing_dir: MissingDir,
}

impl Default for DestinationDirOptions {
    fn default() -> Self {
        Self {
            existing_dir: ExistingDir::Continue,
            missing_dir: MissingDir::Create,
        }
    }
}

impl DestinationDirOptions {
    pub fn set(&mut self, dir: impl Into<DirOption>) {
        match dir.into() {
            DirOption::Existing(existing) => self.existing_dir = existing,
            DirOption::Missing(missing) => self.missing_dir = missing,
        }
    }

    pub fn prepare(&self, dest: &Utf8Path) -> anyhow::Result<bool> {
        if dest.exists() {
            match self.existing_dir {
                ExistingDir::Replace => {
                    dest.replace_dir().map_err(|e| {
                        FileManagerError::new(
                            format!("could not replace dir before copying to it: {dest}"),
                            e,
                        )
                    })?;
                    Ok(true)
                }
                ExistingDir::Continue => Ok(true),
                ExistingDir::Skip => Ok(false),
                ExistingDir::Error => Err(anyhow!("directory exists: {dest}")),
            }
        } else {
            match self.missing_dir {
                MissingDir::Create => {
                    dest.create_dir()
                        .map_err(|source| FileManagerError::new("", source))?;
                    Ok(true)
                }
                MissingDir::Skip => Ok(false),
                MissingDir::Error => Err(anyhow!("missing directory: {dest}")),
            }
        }
    }
}
