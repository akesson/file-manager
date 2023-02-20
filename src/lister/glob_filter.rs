use anyhow::anyhow;
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::FileManagerError;

use super::DirEntry;
trait GlobExt {
    fn fm_new(glob: &str, what: &str) -> crate::Result<Glob>;
}

impl GlobExt for Glob {
    fn fm_new(glob: &str, what: &str) -> crate::Result<Glob> {
        if glob.starts_with('/') {
            return Err(FileManagerError::new(
                format!("invalid {what} glob pattern: {glob}"),
                anyhow!("pattern cannot be absolute (start with '/')"),
            ));
        }

        Glob::new(glob).map_err(|source| {
            FileManagerError::new(format!("invalid glob pattern: {glob}"), source.into())
        })
    }
}

pub struct GlobBuilder {
    exclude: Vec<Glob>,
    exclude_set: GlobSet,
    include: Vec<Glob>,
    include_set: GlobSet,
}

impl Default for GlobBuilder {
    fn default() -> Self {
        Self {
            exclude: Vec::new(),
            exclude_set: GlobSet::empty(),
            include: Vec::new(),
            include_set: GlobSet::empty(),
        }
    }
}

impl GlobBuilder {
    pub fn build(self) -> GlobFilter {
        GlobFilter {
            exclude: self.exclude_set,
            include: self.include_set,
        }
    }

    pub fn exclude(&mut self, pattern: impl AsRef<str>) -> crate::Result<()> {
        self.exclude
            .push(Glob::fm_new(pattern.as_ref(), "exclude")?);
        self.exclude_set = build(&self.exclude, "exclude")?;
        Ok(())
    }

    pub fn include(&mut self, pattern: impl AsRef<str>) -> crate::Result<()> {
        self.include
            .push(Glob::fm_new(pattern.as_ref(), "include")?);
        self.include_set = build(&self.include, "include")?;
        Ok(())
    }
}

fn build(set: &[Glob], what: &str) -> crate::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for glob in set {
        builder.add(glob.clone());
    }
    builder.build().map_err(|source| {
        FileManagerError::new(
            format!(
                "invalid {what} glob pattern set: [{}]",
                set.iter()
                    .map(|g| g.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            source.into(),
        )
    })
}

#[derive(Default)]
pub struct GlobFilter {
    exclude: GlobSet,
    include: GlobSet,
}

impl GlobFilter {
    pub fn is_match(&self, entry: &DirEntry) -> bool {
        let path = entry.relative_path().as_str();
        let include = self.include.is_empty() || self.include.is_match(path);
        let exclude = self.exclude.is_match(path);
        println!("include: {}, exclude: {}, path: {}", include, exclude, path);
        include && !exclude
    }
}
