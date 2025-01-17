use axum::response::Redirect;

use crate::domain::UserSession;

pub async fn get(session: UserSession) -> Result<Redirect, Redirect> {
    match session.user().await {
        Some(_) => UserSession::destroy(&session.session)
            .await
            .map(|()| Redirect::temporary("/"))
            .inspect_err(|error| tracing::error!(?error, "unable to destroy session"))
            .map_err(|_| Redirect::temporary("/err")),
        None => Err(Redirect::temporary("/err")),
    }
}
