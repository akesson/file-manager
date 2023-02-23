use anyhow::Result;
use std::{env::VarError, ffi::OsStr};

pub fn var<K: AsRef<OsStr>>(key: K) -> Result<String> {
    std::env::var(&key).map_err(|var_error| {
        let key = key.as_ref().to_string_lossy();

        match var_error {
            VarError::NotPresent => {
                anyhow::anyhow!("environment variable not found: {key}")
            }
            VarError::NotUnicode(_) => {
                anyhow::anyhow!("environment variable is not valid unicode: {key}")
            }
        }
    })
}
