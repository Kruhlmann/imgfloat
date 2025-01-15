use axum::response::Redirect;

use crate::domain::UserSession;

pub async fn route(session: UserSession) -> Result<Redirect, Redirect> {
    if !session.is_logged_in().await {
        return Err(Redirect::temporary("/err"));
    }
    UserSession::destroy(&session.session)
        .await
        .map(|()| Redirect::temporary("/"))
        .inspect_err(|error| tracing::error!(?error, "unable to destroy session"))
        .map_err(|_| Redirect::temporary("/err"))
}
