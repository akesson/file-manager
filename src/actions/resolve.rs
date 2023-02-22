use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use std::path::Path;

use crate::{helpers::Utf8PathBufExt, FileManagerError};

pub trait PathResolve {
    /// If the path:
    /// - starts with `~`, returns the home directory joined with this path,
    /// - is absolute, returns the same path,
    /// - is relative and starts with any number of `../`, returns the current working dir joined with this path,
    /// - is relative, return the current working dir joined with this path
    ///
    /// # Errors
    /// - Error when retrieving the current working directory
    /// - Error when retrieving the home directory
    fn resolve(&self) -> crate::Result<Utf8PathBuf>;

    /// If the path:
    /// - starts with `~`, returns the home directory joined with this path,
    /// - is absolute, returns the same path,
    /// - is relative, return the base joined with this path
    fn resolve_from(&self, base: impl AsRef<Path>) -> crate::Result<Utf8PathBuf>;
}

impl PathResolve for Utf8Path {
    fn resolve(&self) -> crate::Result<Utf8PathBuf> {
        _resolve(&self).map_err(|source| resolve_error(self, source))
    }

    fn resolve_from(&self, base: impl AsRef<Path>) -> crate::Result<Utf8PathBuf> {
        _resolve_from(&self, &base).map_err(|source| resolve_from_error(self, base, source))
    }
}

pub(crate) fn resolve_error(path: &Utf8Path, source: anyhow::Error) -> FileManagerError {
    FileManagerError::new(format!("could not resolve path {path}"), source)
}

pub(crate) fn resolve_from_error(
    path: &Utf8Path,
    base: impl AsRef<Path>,
    source: anyhow::Error,
) -> FileManagerError {
    FileManagerError::new(
        format!(
            "could not resolve path {path} with base {}",
            base.as_ref().to_string_lossy()
        ),
        source,
    )
}

fn _resolve_from(path: &Utf8Path, base: impl AsRef<Path>) -> anyhow::Result<Utf8PathBuf> {
    if path.starts_with("~") {
        Ok(home_dir()?.join(path.strip_prefix("~").unwrap()))
    } else if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(Utf8PathBuf::from_path(base)?.join(path))
    }
}

fn _resolve(path: &Utf8Path) -> anyhow::Result<Utf8PathBuf> {
    if path.starts_with("~") {
        Ok(home_dir()?.join(path.strip_prefix("~").unwrap()))
    } else if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(cwd()?.join(path))
    }
}

pub(crate) fn home_dir() -> anyhow::Result<Utf8PathBuf> {
    let home = crate::env::var("HOME").context("could not get the user's home directory")?;
    Ok(Utf8PathBuf::from(home))
}

pub(crate) fn cwd() -> anyhow::Result<Utf8PathBuf> {
    let cwd = std::env::current_dir().context("could not get current working directory")?;
    Ok(Utf8PathBuf::from_path_buf(cwd).unwrap())
}

#[test]
fn test_resolve_relative() {
    let path = Utf8PathBuf::from("some/relative");
    let resolved = path.resolve().unwrap();

    let cwd_path = cwd().unwrap().join(path);
    assert_eq!(resolved, cwd_path);
}

#[test]
fn test_resolve_absolute() {
    let path = Utf8PathBuf::from("/some/absolute");
    let resolved = path.resolve().unwrap();
    assert_eq!(resolved, path);
}

#[test]
fn test_resolve_home() {
    let path = Utf8PathBuf::from("~/some/home");
    let resolved = path.resolve().unwrap();

    let home_path = home_dir().unwrap().join("some/home");
    assert_eq!(resolved, home_path);
}

#[test]
fn test_resolve_home_dir_error() {
    let home_err = crate::env::var("HOME_WRONG")
        .context("could not get the user's home directory")
        .unwrap_err();

    let err = resolve_error(&Utf8PathBuf::from("my/relative/path"), home_err);

    insta::assert_snapshot!(format!("{err}"), @r###"
    Error: could not resolve path my/relative/path
           ↳ could not get the user's home directory
             ↳ environment variable not found: HOME_WRONG
    "###);

    insta::assert_snapshot!(format!("{err:#}"), @"could not resolve path my/relative/path ── could not get the user's home directory ── environment variable not found: HOME_WRONG");

    insta::assert_snapshot!(format!("{err:?}"), @r###"
    ⏎
    Error: could not resolve path my/relative/path
           ↳ could not get the user's home directory
             ↳ environment variable not found: HOME_WRONG
    "###);

    insta::assert_snapshot!(format!("{err:#?}"), @r###"
    FileManagerError {
        message: "could not resolve path my/relative/path",
        source: "could not get the user's home directory",
    }
    "###);
}
