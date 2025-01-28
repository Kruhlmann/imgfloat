use std::{ffi::OsStr, path::PathBuf};

pub struct EnvVar<Inner>(pub Option<Inner>);

impl<Inner> EnvVar<Inner> {
    pub fn unwrap(self) -> Inner {
        self.0.unwrap()
    }
}

impl EnvVar<String> {
    pub fn new(key: &str) -> EnvVar<String> {
        match std::env::var(key) {
            Ok(value) => EnvVar(Some(value)),
            Err(error) => {
                tracing::error!(?error, ?key, "unable to read env var");
                EnvVar(None)
            }
        }
    }

    pub fn with_default_value(self, default: impl Into<String>) -> EnvVar<String> {
        match self.0 {
            Some(_) => self,
            None => EnvVar(Some(default.into())),
        }
    }
}

impl<Inner> EnvVar<Inner>
where
    Inner: std::fmt::Debug,
{
    pub fn ensure_non_empty(self) -> Self
    where
        Inner: AsRef<str>,
    {
        if self.0.as_ref().unwrap().as_ref().is_empty() {
            tracing::error!("env var was empty");
            return EnvVar(None);
        }
        self
    }

    pub fn ensure_file(self) -> Self
    where
        Inner: AsRef<OsStr>,
    {
        let path = PathBuf::from(self.0.as_ref().expect("No value in EnvVar"));
        if !path.is_file() {
            tracing::error!(?path, "file not found");
            return EnvVar(None);
        }
        self
    }

    pub fn ensure_directory(self) -> Self
    where
        Inner: AsRef<OsStr>,
    {
        let path = PathBuf::from(OsStr::new(self.0.as_ref().expect("No value in EnvVar")));
        if !path.is_dir() {
            tracing::error!(?path, "directory not found");
            return EnvVar(None);
        }
        self
    }
}

impl<Inner> EnvVar<Inner>
where
    Inner: std::fmt::Debug + AsRef<str>,
{
    pub fn ensure_parsable<T>(self) -> Self
    where
        T: std::str::FromStr<Err: std::fmt::Debug> + std::fmt::Debug,
    {
        match self.parse::<T>().0 {
            Some(_) => self,
            None => EnvVar(None),
        }
    }

    pub fn parse<T>(&self) -> EnvVar<T>
    where
        T: std::str::FromStr<Err: std::fmt::Debug> + std::fmt::Debug,
    {
        match self.0.as_ref().unwrap().as_ref().parse() {
            Ok(parsed) => EnvVar(Some(parsed)),
            Err(error) => {
                tracing::error!(
                    ?error,
                    value = ?self.0,
                    target = std::any::type_name::<T>(),
                    "couldn't parse env variable"
                );
                EnvVar(None)
            }
        }
    }
}
