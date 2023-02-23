mod destination;
mod dir;
mod filter;

use camino::{Utf8Path, Utf8PathBuf};
pub use destination::{Destination, DestinationMut};
pub use dir::{DirOptions, DirOptionsMut};
pub use filter::{FilterOptionMut, FilterOptions};

pub struct Build<T> {
    pub(crate) path: Utf8PathBuf,
    pub(crate) build: T,
}

impl<T> Build<T>
where
    T: PathNew,
{
    pub fn new(path: impl AsRef<Utf8Path>) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            build: T::new(path),
        }
    }
}

pub trait PathNew {
    fn new(path: impl AsRef<Utf8Path>) -> Self;
}
