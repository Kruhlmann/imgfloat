use dioxus::prelude::*;

use cookie::Cookie;

pub struct SessionCookie<'a>(pub Cookie<'a>);

impl SessionCookie<'_> {
    pub fn try_from_header_map(headers: http::HeaderMap) -> Result<Self, ServerFnError> {
        let cookie = headers
            .get(http::header::COOKIE)
            .and_then(|header| header.to_str().ok())
            .map(|cookie_str| cookie_str.trim_matches('"'))
            .map(|cookie_str| Cookie::parse_encoded(cookie_str))
            .ok_or_else(|| ServerFnError::new("missing cookie"))?
            .inspect_err(|error| tracing::error!(?error, "invalid cookie"))
            .map_err(|_| ServerFnError::new("invalid cookie"))?
            .into_owned();
        Ok(Self(cookie))
    }
}
