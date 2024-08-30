use lib_routes::nested_route::NestedRoute;
use std::sync::{Arc, Mutex};

use self::{auth_route::AuthRoute, hello_world::HelloWorldRoute, users_route::UserRoute};
use crate::{
    middleware::{
        auth_mw::{ctx_resolver, validate_auth},
        logger_mw::logger,
    },
    models,
    services::ndarray::NDArrayAppState,
};

use axum::{
    middleware::{from_fn, map_response},
    Router,
};
use content_route::ContentRoute;
use exercise_preset_route::ExercisePresetRoute;
use sqlx::{Pool, Postgres};
use tower_cookies::CookieManagerLayer;

mod auth_route;
mod content_route;
mod exercise_preset_route;
mod hello_world;
mod users_route;

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub s3_client: aws_sdk_s3::Client,
    pub ndarray_app_state: Arc<Mutex<NDArrayAppState>>,
}

pub fn create_routes(app_state: AppState) -> Router {
    Router::new()
        .nest(HelloWorldRoute::PATH, HelloWorldRoute::router())
        .nest(UserRoute::PATH, UserRoute::router())
        .nest(ContentRoute::PATH, ContentRoute::router())
        .layer(from_fn(validate_auth))
        .nest(ExercisePresetRoute::PATH, ExercisePresetRoute::router())
        .nest(AuthRoute::PATH, AuthRoute::router())
        .layer(from_fn(ctx_resolver))
        .layer(map_response(logger))
        .layer(CookieManagerLayer::new())
        .with_state(app_state)
}
