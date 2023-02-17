use anyhow::{bail, Context};
use std::{fs::File, path::Path};

pub trait FileExt {
    fn open_read(path: impl AsRef<Path>) -> anyhow::Result<File>;
    fn open_write(path: impl AsRef<Path>, overwrite: bool) -> anyhow::Result<File>;
}

impl FileExt for File {
    fn open_read(path: impl AsRef<Path>) -> anyhow::Result<File> {
        File::open(&path)
            .with_context(|| format!("could not open file {}", path.as_ref().to_string_lossy()))
    }

    /// Open a file for writing.
    /// - path: the path to the file
    /// - overwrite: if file exists, overwrite it
    fn open_write(path: impl AsRef<Path>, overwrite: bool) -> anyhow::Result<File> {
        let path = path.as_ref();
        if overwrite {
            File::create(path).with_context(|| {
                format!("could not open file for writing {}", path.to_string_lossy())
            })
        } else {
            if path.exists() {
                bail!("file {} already exists", path.to_string_lossy());
            }
            File::create(path)
                .with_context(|| format!("could not create file {}", path.to_string_lossy()))
        }
    }
}
