use camino::Utf8Path;
use std::path::{Path, PathBuf};

pub trait PathList {
    fn list(&self) -> ListBuilder;
}

impl PathList for Utf8Path {
    fn list(&self) -> ListBuilder {
        ListBuilder::new(self.to_path_buf())
    }
}

pub struct ListBuilder {
    path: PathBuf,
}

impl ListBuilder {
    fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        Self { path }
    }

    #[cfg(feature = "ascii")]
    pub fn ascii(self) -> Result<String, crate::FileManagerError> {
        _ascii(&self.path).map_err(|source| {
            crate::FileManagerError::new(
                format!("Could not create ascii listing of {:?}", self.path),
                source,
            )
        })
    }
}

#[cfg(feature = "ascii")]
use crate::helpers::Tree;

#[cfg(feature = "ascii")]
pub(crate) fn _ascii(path: &Path) -> Result<String, anyhow::Error> {
    Ok(format!("{}", list_ascii(&path)?))
}

#[cfg(feature = "ascii")]
fn list_ascii(path: impl AsRef<Path>) -> Result<Tree, anyhow::Error> {
    use crate::helpers::PathExt;
    use anyhow::bail;

    let path = path.as_ref();
    let Some(name) = path.file_name() else {
        bail!("Could not get file name from path {path:?}");
    };
    let name = name.to_string_lossy().to_string();
    if path.is_dir() {
        let entries = path.fm_read_dir()?;

        let mut content = Vec::new();

        for entry in entries {
            let Ok(entry) = entry else {
              panic!("")
            };
            let path = entry.path();
            content.push(list_ascii(path)?);
        }
        Ok(Tree::node(name, content))
    } else {
        Ok(Tree::leaf(name))
    }
}
