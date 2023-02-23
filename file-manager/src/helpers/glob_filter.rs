use super::GlobSet;

#[derive(Clone)]
pub struct GlobFilter {
    pub debug: bool,
    pub exclude: GlobSet,
    pub include: GlobSet,
}

impl Default for GlobFilter {
    fn default() -> Self {
        Self {
            debug: false,
            exclude: GlobSet::new("exclude"),
            include: GlobSet::new("include"),
        }
    }
}

impl GlobFilter {
    pub fn is_match(&self, entry: &str) -> bool {
        let include = self.include.is_empty() || self.include.is_match(entry);
        if !include {
            if self.debug {
                println!("[filter] not included: {}", entry);
            }
            return false;
        }
        let exclude = self.exclude.is_match(entry);
        if exclude {
            if self.debug {
                println!("[filter] excluded: {}", entry);
            }
            return false;
        }
        true
    }
}
