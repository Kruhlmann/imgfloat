use std::path::Path;

pub struct EnvVar(pub String);

impl EnvVar {
    pub fn new(key: &str) -> Self {
        match std::env::var(key).expect(&format!("{key} unset")).as_str() {
            "" => panic!("{key} unset"),
            str => Self(str.to_string()),
        }
    }

    pub fn ensure_file(self) -> Self {
        let path = Path::new(&self.0);
        if !path.is_file() {
            tracing::error!(?path, "file not found");
            panic!();
        }
        self
    }

    pub fn ensure_directory(self) -> Self {
        let path = Path::new(&self.0);
        if !path.is_dir() {
            tracing::error!(?path, "directory not found");
            panic!();
        }
        self
    }
}
