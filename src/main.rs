use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, time::Duration};

mod lib;
mod models;
mod repositories;
mod view_models;

#[tokio::main]
async fn main() {
    // let pool = create_pool().await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Could not run migrations");

    let router = repositories::create_routes(pool);

    let addr = "0.0.0.0:3000";
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
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("Could not connect to the database");
    pool
}

async fn create_diesel_pool() {
    dotenv().expect(".env file not found");
    #[allow(unused_assignments)]
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found in .env");

    // let connection =
}
