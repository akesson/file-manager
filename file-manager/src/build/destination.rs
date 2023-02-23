use crate::build::Build;
use crate::helpers::{DestinationDirOptions, DestinationFileOptions, DirOption, FileOption};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::HashMap;
use std::fs::File;

#[derive(Default)]
pub struct Destination {
    pub debug: bool,
    pub file: DestinationFileOptions,
    pub dir: DestinationDirOptions,
}

impl Destination {
    pub fn to_dir(&self, dest_root: &Utf8Path) -> DestinationCheck {
        DestinationCheck {
            debug: self.debug,
            dest_root: dest_root.to_path_buf(),
            dir_cache: HashMap::new(),
            file: self.file.clone(),
            dir: self.dir.clone(),
        }
    }
}
pub struct DestinationCheck {
    debug: bool,
    dest_root: Utf8PathBuf,
    dir_cache: HashMap<Utf8PathBuf, bool>,
    file: DestinationFileOptions,
    dir: DestinationDirOptions,
}

impl DestinationCheck {
    pub fn enter_dir(&mut self, relative_dir: &Utf8Path) -> anyhow::Result<bool> {
        assert!(
            !relative_dir.is_absolute(),
            "relative_dir must be relative: {relative_dir}"
        );
        let mut dir = self.dest_root.clone();
        for component in relative_dir.components() {
            dir = dir.join(component);
            if !self.enter_dir_elem(&dir)? {
                if self.debug {
                    println!("[destination]");
                }
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn enter_dir_elem(&mut self, dir: &Utf8Path) -> anyhow::Result<bool> {
        if let Some(enter) = self.dir_cache.get(dir) {
            Ok(*enter)
        } else {
            let enter = self.dir.prepare(dir)?;
            self.dir_cache.insert(dir.to_path_buf(), enter);
            Ok(enter)
        }
    }

    pub fn get_file(&mut self, file: &Utf8Path) -> anyhow::Result<Option<File>> {
        if let Some(dir) = file.parent() {
            if !self.enter_dir(dir)? {
                return Ok(None);
            }
        }
        self.file.get_file(file)
    }

    pub fn get_file_path(&mut self, file: &Utf8Path) -> anyhow::Result<Option<Utf8PathBuf>> {
        if let Some(dir) = file.parent() {
            if !self.enter_dir(dir)? {
                return Ok(None);
            }
        }
        self.file.get_path(file)
    }
}

pub trait DestinationMut {
    fn destination_options_mut(&mut self) -> &mut Destination;
}

impl<T> Build<T>
where
    T: DestinationMut,
{
    pub fn destination_file(&mut self, file: impl Into<FileOption>) -> &mut Self {
        self.build.destination_options_mut().file.set(file);
        self
    }

    pub fn destination_dir(&mut self, dir: impl Into<DirOption>) -> &mut Self {
        self.build.destination_options_mut().dir.set(dir);
        self
    }
}
