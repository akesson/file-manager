use std::error::Error;

pub struct FileManagerError {
    message: String,
    source: Option<anyhow::Error>,
}

impl std::fmt::Display for FileManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.message)?;
            let mut source = self.source().unwrap();

            loop {
                write!(f, " ── {}", source)?;
                let Some(next) = source.source() else {
                    break;
                };

                source = next;
            }
        } else {
            writeln!(f, "Error: {}", self.message)?;
            let mut source = self.source().unwrap();
            let mut indent = 0;

            loop {
                writeln!(f, "{}       ↳ {}", "  ".repeat(indent), source)?;
                let Some(next) = source.source() else {
                    break;
                };

                source = next;
                indent += 1;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for FileManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            if let Some(source) = self.source() {
                f.debug_struct("FileManagerError")
                    .field("message", &self.message)
                    .field("source", &source.to_string())
                    .finish()
            } else {
                f.debug_struct("FileManagerError")
                    .field("message", &self.message)
                    .finish()
            }
        } else {
            // this is what is output when doing an .unwrap() on a Result
            f.write_str(&format!("⏎\n{self}"))
        }
    }
}

impl std::error::Error for FileManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.source.as_ref() {
            Some(source) => Some(source.as_ref()),
            None => None,
        }
    }
}

impl FileManagerError {
    pub fn new(message: impl AsRef<str>, source: anyhow::Error) -> Self {
        Self {
            message: message.as_ref().to_string(),
            source: Some(source),
        }
    }
}

impl<T> From<T> for FileManagerError
where
    T: AsRef<str>,
{
    fn from(message: T) -> Self {
        Self {
            message: message.as_ref().to_string(),
            source: None,
        }
    }
}

impl From<FileManagerError> for String {
    fn from(err: FileManagerError) -> Self {
        format!("{err:#}")
    }
}
