use crate::helpers::GlobFilter;

use super::Build;

pub struct FilterOptions {
    #[cfg(feature = "glob")]
    pub(crate) glob: GlobFilter,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            #[cfg(feature = "glob")]
            glob: GlobFilter::default(),
        }
    }
}

pub trait FilterOptionMut {
    fn filter_options(&mut self) -> &mut FilterOptions;
}

impl<T> Build<T>
where
    T: FilterOptionMut,
{
    #[cfg(feature = "glob")]
    pub fn filter_exclude(mut self, glob: &str) -> crate::Result<Self> {
        self.build.filter_options().glob.exclude.add(glob)?;
        Ok(self)
    }

    #[cfg(feature = "glob")]
    pub fn filter_include(mut self, glob: &str) -> crate::Result<Self> {
        self.build.filter_options().glob.include.add(glob)?;
        Ok(self)
    }
}
