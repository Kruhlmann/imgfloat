use crate::twitch::TwitchUser;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Username(pub String);

impl From<String> for Username {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<TwitchUser> for Username {
    fn from(value: TwitchUser) -> Self {
        Self(value.login.clone())
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<String> for Username {
    fn into(self) -> String {
        self.0
    }
}
