use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, time::Duration};
use tower_http::cors::{Any, CorsLayer};

mod libs;
mod middleware;
mod models;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    let pool = create_pool().await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Could not run migrations");

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);
    let router = routes::create_routes(pool).layer(cors);

    let addr = "0.0.0.0:3001";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect(&format!("Could not listen at {}", addr));

    println!("Serving on {}", addr);
    axum::serve(listener, router)
        .await
        .expect("Could not serve axum app");
}

async fn create_pool() -> Pool<Postgres> {
    dotenv().expect(".env file not found");
    #[allow(unused_assignments)]
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found in .env");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("Could not connect to the database");
    pool
}
