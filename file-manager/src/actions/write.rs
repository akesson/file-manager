use crate::{helpers::FileExt, FileManagerError};
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub trait PathWrite {
    fn write(&self) -> Writer;
}

impl PathWrite for Utf8Path {
    fn write(&self) -> Writer {
        Writer::new(self.to_path_buf())
    }
}

pub struct Writer {
    path: Utf8PathBuf,
    overwrite: bool,
}

impl Writer {
    fn new(path: Utf8PathBuf) -> Self {
        Self {
            path,
            overwrite: true,
        }
    }

    pub fn no_overwrite(mut self) -> Self {
        self.overwrite = false;
        self
    }

    pub fn string(self, s: impl AsRef<str>) -> crate::Result<Utf8PathBuf> {
        write(&self, s.as_ref().as_bytes(), "string")
    }

    pub fn bytes(self, contents: impl AsRef<[u8]>) -> crate::Result<Utf8PathBuf> {
        write(&self, contents, "bytes")
    }

    pub fn zip(self) -> ZipWriter {
        ZipWriter { writer: self }
    }
}

pub struct ZipWriter {
    pub writer: Writer,
}

pub fn write(
    builder: &Writer,
    contents: impl AsRef<[u8]>,
    what: &str,
) -> crate::Result<Utf8PathBuf> {
    _write(builder, contents).map_err(|source| {
        FileManagerError::new(
            format!("Could not write {what} to {}", builder.path),
            source.into(),
        )
    })
}

pub fn _write(builder: &Writer, contents: impl AsRef<[u8]>) -> anyhow::Result<Utf8PathBuf> {
    let file = File::open_write(&builder.path, builder.overwrite)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(contents.as_ref())?;

    writer.flush()?;
    Ok(builder.path.to_path_buf())
}
