use crate::{helpers::FileExt, FileManagerError};
use camino::{Utf8Path, Utf8PathBuf};
use std::{fs::File, io::Read};

pub trait PathRead {
    fn read(&self) -> Reader;
}

impl PathRead for Utf8Path {
    fn read(&self) -> Reader {
        Reader::new(self.clone())
    }
}

pub struct Reader {
    path: Utf8PathBuf,
}

impl Reader {
    fn new(path: impl Into<Utf8PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn string(self) -> crate::Result<String> {
        read_string(&self)
    }
}

fn read_string(reader: &Reader) -> crate::Result<String> {
    read(&reader, |bytes| Ok(String::from_utf8(bytes)?), "String")
}

fn read<F, T>(reader: &Reader, convert: F, what: &str) -> crate::Result<T>
where
    F: FnOnce(Vec<u8>) -> anyhow::Result<T>,
{
    let bytes = _read(reader).map_err(|source| {
        FileManagerError::new(format!("could not read {}", &reader.path), source)
    })?;
    convert(bytes).map_err(|source| {
        FileManagerError::new(
            format!(
                "could not convert the contents of {} to {what}",
                &reader.path
            ),
            source,
        )
    })
}
fn _read(reader: &Reader) -> anyhow::Result<Vec<u8>> {
    let file = File::open_read(&reader.path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}
