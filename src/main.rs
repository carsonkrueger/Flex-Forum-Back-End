use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, time::Duration};

mod routes;

#[tokio::main]
async fn main() {
    let pool = create_pool().await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Could not run migrations");

    let router = routes::create_routes(pool);

    let addr = "0.0.0.0:3000";
    println!("Serving on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect(&format!("Could not listen at {}", addr));

    axum::serve(listener, router)
        .await
        .expect("Could not serve axum app");
}

async fn create_pool() -> Pool<Postgres> {
    dotenv().expect(".env file not found");
    #[allow(unused_assignments)]
    let mut db_url = env::var("PROD_DATABASE_URL").expect("PROD_DATABASE_URL not found in .env");

    // if running debug build, use localhost connection to database
    #[cfg(debug_assertions)]
    {
        db_url = env::var("DEBUG_DATABASE_URL").expect("DEBUG_DATABASE_URL not found in .env");
    }

    let pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("Could not connect to the database");
    pool
}
