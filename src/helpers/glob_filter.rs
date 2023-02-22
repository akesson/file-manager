use super::GlobSet;

pub struct GlobFilter {
    pub exclude: GlobSet,
    pub include: GlobSet,
}

impl Default for GlobFilter {
    fn default() -> Self {
        Self {
            exclude: GlobSet::new("exclude"),
            include: GlobSet::new("include"),
        }
    }
}

impl GlobFilter {
    pub fn is_match(&self, entry: &str) -> bool {
        let include = self.include.is_empty() || self.include.is_match(entry);
        let exclude = self.exclude.is_match(entry);
        include && !exclude
    }
}
