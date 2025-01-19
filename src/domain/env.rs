pub struct EnvVar(pub String);

impl EnvVar {
    pub fn new(key: &str) -> Self {
        Self(std::env::var(key).expect(&format!("{key} unset")))
    }
}
