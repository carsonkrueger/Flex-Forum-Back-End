use axum::extract::State;
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, time::Duration};

mod routes;

#[tokio::main]
async fn main() {
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

    // let router = routes::create_routes(pool);

    let router = axum::Router::new()
        .route("/", axum::routing::get(hello_world))
        .with_state(pool);

    // let routes = vec![("/", get(hello_world).get(hello_world))];

    // for (path, handlers) in routes {
    //     app = app.route(path, handlers);
    // }

    // let app = Router::new().route("/", get(hello_world)).with_state(pool);
    // Router::new().route("/")

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect(&format!("Could not listen at {}", addr));

    axum::serve(listener, router)
        .await
        .expect("Could not serve axum app");
}

async fn hello_world() -> String {
    "hello world".to_string()
}
