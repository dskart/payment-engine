use crate::app::{self, SanitizedError};
use rocket::{fairing::AdHoc, Build, Rocket};
use std::sync::Arc;

pub mod error;
pub use error::*;
pub mod session;
pub use session::*;
pub mod clients;
pub use clients::*;
pub mod transactions;
pub use transactions::*;

#[rocket::get("/healthz")]
fn healthz() -> Result<String, SanitizedError> {
    Ok("OK".to_string())
}

#[rocket::get("/")]
fn index() -> Result<String, SanitizedError> {
    Ok("Welcome to the payment-engine API".to_string())
}

pub trait Route {
    fn stage() -> AdHoc;
}

pub struct State {
    pub app: app::App,
    pub logger: slog::Logger,
}

impl State {
    fn fetch<P: rocket::Phase>(rocket: &Rocket<P>) -> Option<Arc<Self>> {
        if let Some(state) = rocket.state::<Arc<State>>() {
            return Some(state.clone());
        }
        None
    }
}

#[derive(Clone)]
pub struct API {
    state: Arc<State>,
}

impl API {
    pub fn new(logger: slog::Logger, app: app::App) -> Self {
        return API {
            state: Arc::new(State { app, logger }),
        };
    }

    pub fn rocket(&self, port: u16) -> crate::Result<Rocket<Build>> {
        let mut provider = rocket::Config::default();
        provider.port = port;

        let r = rocket::custom(provider)
            .manage(self.state.clone())
            .mount("/", rocket::routes![healthz, index])
            .attach(ClientRoute::stage())
            .attach(TransactionRoute::stage());

        return Ok(r);
    }
}

#[cfg(test)]
impl API {
    pub async fn new_test_api() -> Self {
        Self::new_test_api_with_app_config(|_| {}).await
    }

    pub fn test_rocket(&self) -> crate::Result<Rocket<Build>> {
        self.rocket(8080)
    }

    pub async fn test_rocket_client(&self) -> rocket::local::asynchronous::Client {
        let client = rocket::local::asynchronous::Client::tracked(self.test_rocket().unwrap()).await.unwrap();
        return client;
    }

    pub async fn new_test_api_with_app_config<F: FnOnce(&mut app::Config)>(app_config: F) -> Self {
        use crate::store;

        let mut config = app::Config {
            store: store::Config {
                in_memory: true,
                ..Default::default()
            },
            ..Default::default()
        };
        app_config(&mut config);
        let a = app::App::new_with_config(config).await.expect("failed to create test app");

        return Self::new(API::test_logger(), a);
    }

    pub fn test_logger() -> slog::Logger {
        use slog::Drain;
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().filter_level(slog::Level::Warning).fuse();
        slog::Logger::root(drain, o!())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::http::Status;

    #[rocket::async_test]
    async fn test_healthz() {
        let api = API::new_test_api().await;
        let client = api.test_rocket_client().await;
        let req = client.get(rocket::uri!(healthz()));

        let (r1, r2) = rocket::tokio::join!(req.clone().dispatch(), req.dispatch());
        assert_eq!(r1.status(), r2.status());
        assert_eq!(r1.status(), Status::Ok);

        let (s1, s2) = (r1.into_string().await, r2.into_string().await);
        assert_eq!(s1, s2);
        assert_eq!(s1.unwrap(), "OK".to_string());
    }
}
