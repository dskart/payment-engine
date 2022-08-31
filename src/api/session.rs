use crate::{api, app};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use std::sync::Arc;

pub struct Context {
    state: Arc<api::State>,
    session: Arc<app::DetachedSession>,
}

impl Context {
    pub fn new(state: Arc<api::State>, session: app::DetachedSession) -> Self {
        let session = Arc::new(session);
        Self {
            state: state.clone(),
            session: session.clone(),
        }
    }

    pub fn session(&self) -> app::Session {
        let app = &self.state.app;
        (*self.session).clone().attach(app)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Context {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match api::State::fetch(req.rocket()) {
            Some(state) => {
                let app = &state.app;
                let logger = &state.logger;
                let session = app.new_session(logger.clone());
                return Outcome::Success(Context::new(state.clone(), session.detach()));
            }
            None => Outcome::Failure((Status::InternalServerError, "internal error".to_string())),
        }
    }
}
