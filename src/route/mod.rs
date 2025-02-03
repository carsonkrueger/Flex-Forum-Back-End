use crate::{
    middleware::{
        auth_mw::{ctx_resolver, validate_auth},
        logger_mw::logger,
    },
    model,
    services::ndarray::NDArrayAppState,
};
use axum::{
    middleware::{from_fn, map_response},
    Router,
};
use private_routes::{
    content::ContentRoute, exercise_preset::ExercisePresetRoute, users::UserRoute,
};
use public_routes::{auth::AuthRoute, hello_world::HelloWorldRoute};
use sqlx::{Pool, Postgres};
use std::sync::{Arc, Mutex};
use tower_cookies::CookieManagerLayer;

pub mod error;
mod private_routes;
mod public_routes;

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub s3_client: aws_sdk_s3::Client,
    pub ndarray_app_state: Arc<Mutex<NDArrayAppState>>,
}

pub trait NestedRoute<S> {
    const PATH: &'static str;
    fn router() -> Router<S>;
}

pub fn create_routes(app_state: AppState) -> Router {
    Router::new()
        // v v PRIVATE ROUTES v v
        .nest(HelloWorldRoute::PATH, HelloWorldRoute::router())
        .nest(UserRoute::PATH, UserRoute::router())
        .nest(ContentRoute::PATH, ContentRoute::router())
        // ^ ^ PRIVATE ROUTES ^ ^
        .layer(from_fn(validate_auth))
        // v v PUBLIC ROUTES v v
        .nest(ExercisePresetRoute::PATH, ExercisePresetRoute::router())
        .nest(AuthRoute::PATH, AuthRoute::router())
        // ^ ^ PUBLIC ROUTES ^ ^
        .layer(from_fn(ctx_resolver))
        .layer(map_response(logger))
        .layer(CookieManagerLayer::new())
        .with_state(app_state)
}
