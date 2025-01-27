use std::sync::Arc;

use imgfloat::domain::UserSession;
use tower_sessions::{MemoryStore, Session};

pub struct EmptySession(pub UserSession);

impl EmptySession {
    pub fn new() -> Self {
        let session = Session::new(None, Arc::new(MemoryStore::default()), None);
        Self(UserSession {
            session,
            user: None,
        })
    }
}
