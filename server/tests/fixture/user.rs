use std::sync::Arc;

use imgfloat::{domain::UserSession, models::User, twitch::TwitchUser};
use tower_sessions::{MemoryStore, Session};

pub struct TestUser {
    username: String,
}

impl TestUser {
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
        }
    }
    pub fn as_twitch_user(&self) -> TwitchUser {
        TwitchUser {
            id: "".to_string(),
            login: self.username.clone(),
            display_name: "".to_string(),
            r#type: "".to_string(),
            broadcaster_type: "".to_string(),
            description: "".to_string(),
            profile_image_url: "".to_string(),
            offline_image_url: "".to_string(),
            created_at: "".to_string(),
        }
    }

    pub fn as_db_user(&self) -> User {
        User {
            username: self.username.clone(),
        }
    }

    pub fn create_session(&self) -> UserSession {
        let session = Session::new(None, Arc::new(MemoryStore::default()), None);
        UserSession {
            user: Some(self.as_twitch_user()),
            session,
        }
    }
}
