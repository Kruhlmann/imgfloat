#[derive(serde::Deserialize)]
pub struct AuthCallbackQuery {
    pub code: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub struct AuthCallbackSuccessQuery {
    pub code: String,
}

#[derive(Debug)]
pub struct AuthCallbackFailureQuery {
    pub error: String,
    pub error_description: String,
}

impl AuthCallbackQuery {
    pub fn as_success(&self) -> AuthCallbackSuccessQuery {
        AuthCallbackSuccessQuery {
            code: self.code.clone().unwrap(),
        }
    }

    pub fn as_failure(&self) -> AuthCallbackFailureQuery {
        AuthCallbackFailureQuery {
            error: self.error.clone().unwrap(),
            error_description: self.error_description.clone().unwrap(),
        }
    }
}
