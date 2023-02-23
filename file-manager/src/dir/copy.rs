use std::{fs, path::Path};

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

use crate::{
    build::{
        Build, Destination, DestinationMut, DirOptions, DirOptionsMut, FilterOptionMut,
        FilterOptions, PathNew,
    },
    helpers::Utf8PathBufExt,
    FileManagerError,
};

pub trait DirCopy {
    fn dir_copy(&self) -> Build<CopyOptions>;
}

impl DirCopy for Utf8Path {
    fn dir_copy(&self) -> Build<CopyOptions> {
        Build::new(&self)
    }
}

pub struct CopyOptions {
    dir_options: DirOptions,
    filter_options: FilterOptions,
    destination_options: Destination,
}

impl PathNew for CopyOptions {
    fn new(path: impl AsRef<Utf8Path>) -> Self {
        CopyOptions {
            dir_options: DirOptions::new(path.as_ref()),
            filter_options: FilterOptions::default(),
            destination_options: Destination::default(),
        }
    }
}

impl DestinationMut for CopyOptions {
    fn destination_options_mut(&mut self) -> &mut Destination {
        &mut self.destination_options
    }
}

impl DirOptionsMut for CopyOptions {
    fn dir_options(&mut self) -> &mut DirOptions {
        &mut self.dir_options
    }
}

impl FilterOptionMut for CopyOptions {
    fn filter_options(&mut self) -> &mut FilterOptions {
        &mut self.filter_options
    }
}

impl Build<CopyOptions> {
    pub fn debug(mut self) -> Self {
        self.build.filter_options.glob.debug = true;
        self.build.destination_options.debug = true;
        self
    }

    pub fn to_dir(self, dir: impl AsRef<Path>) -> crate::Result<Utf8PathBuf> {
        self._to_dir(&dir).map_err(|source| {
            FileManagerError::new(
                format!(
                    "Could not copy directory {} to {:?}",
                    &self.path,
                    dir.as_ref()
                ),
                source,
            )
        })
    }

    fn _to_dir(&self, dir: impl AsRef<Path>) -> anyhow::Result<Utf8PathBuf> {
        let dir = Utf8PathBuf::from_path(&dir)?;
        let mut dest = self.build.destination_options.to_dir(&dir);
        let input = self
            .build
            .dir_options
            .filtered_iter(self.build.filter_options.clone());
        for entry in input {
            let entry = entry?;
            if entry.file_type().is_dir() {
                dest.enter_dir(entry.relative_path())?;
            } else {
                if let Some(file) = dest.get_file_path(entry.relative_path())? {
                    fs::copy(&entry.path, &file)
                        .context(format!("could not copy file {file} to {}", &entry.path))?;
                }
            }
        }
        Ok(dir)
    }
}
