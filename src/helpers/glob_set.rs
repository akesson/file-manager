use crate::FileManagerError;
use anyhow::anyhow;

#[derive(Clone)]
pub struct GlobSet {
    name: String,
    patterns: Vec<globset::Glob>,
    set: globset::GlobSet,
}

impl GlobSet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            patterns: Vec::new(),
            set: globset::GlobSet::empty(),
        }
    }
}

impl GlobSet {
    pub fn add(&mut self, glob: &str) -> crate::Result<()> {
        let name = &self.name;
        if glob.starts_with('/') {
            return Err(FileManagerError::new(
                format!("invalid {name} glob pattern: {glob}"),
                anyhow!("pattern cannot be absolute (start with '/')"),
            ));
        }

        let pattern = globset::Glob::new(glob).map_err(|source| {
            FileManagerError::new(
                format!("invalid {name} glob pattern: {glob}"),
                source.into(),
            )
        })?;

        self.patterns.push(pattern);
        self.set = build(&self.patterns, name)?;
        Ok(())
    }

    pub fn is_match(&self, path: &str) -> bool {
        self.set.is_match(path)
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
}

fn build(set: &[globset::Glob], name: &str) -> crate::Result<globset::GlobSet> {
    let mut builder = globset::GlobSetBuilder::new();
    for glob in set {
        builder.add(glob.clone());
    }
    builder.build().map_err(|source| {
        FileManagerError::new(
            format!(
                "invalid {name} glob pattern set: [{}]",
                set.iter()
                    .map(|g| g.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            source.into(),
        )
    })
}
