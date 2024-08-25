use aws_sdk_s3::config::Credentials;
use dotenvy::dotenv;
use models::{
    content_model::ContentModel, interactions_matrix_model::InteractionsMatrixModel,
    user_model::UserModel,
};
use routes::AppState;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, sync::Arc, time::Duration};
use tensorflow::Tensor;
use tower_http::cors::{Any, CorsLayer};

mod libs;
mod middleware;
mod models;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    let pool = create_pool().await;
    let s3_client = create_s3_client().await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Could not run migrations");

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let (u_embeddings, v_embeddings) = load_embeddings(&pool, 10).await;

    let app_state = AppState {
        pool,
        s3_client,
        u_embeddings: Arc::new(u_embeddings),
        v_embeddings: Arc::new(v_embeddings),
    };
    let router = routes::create_routes(app_state).layer(cors);

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

async fn create_s3_client() -> aws_sdk_s3::Client {
    let access_key = env::var("ACCESS_KEY_ID").expect("ACCESS_KEY_ID not found in .env");
    let secret_access_key =
        env::var("SECRET_ACCESS_KEY").expect("SECRET_ACCESS_KEY not found in .env");
    let credentials = Credentials::new(access_key, secret_access_key, None, None, "FlexForum");
    let config = aws_config::from_env()
        .region("us-east-1")
        .credentials_provider(credentials)
        .load()
        .await;
    aws_sdk_s3::Client::new(&config)
}

/// -> (u, v)
async fn load_embeddings(pg_pool: &Pool<Postgres>, k: u64) -> (Tensor<f32>, Tensor<f32>) {
    let u_count = models::base::count::<UserModel>(pg_pool)
        .await
        .expect("Could not count users") as u64;
    let v_count = models::base::count::<ContentModel>(pg_pool)
        .await
        .expect("Could not count posts") as u64;

    let u = Tensor::new(&[u_count, k]);
    let v = Tensor::new(&[v_count, k]);

    let join_query = sqlx::query_as::<Postgres, InteractionsMatrixModel>(
        "
        SELECT
            u.id as user_id,
            p.id as post_id,
            u.username
                FROM post_management.posts p
                JOIN user_management.users u
                    ON p.username = u.username;
            ",
    )
    .fetch_all(pg_pool)
    .await
    .expect("Could not query for interactions matrix model");

    let all_users = sqlx::query_as::<_, UserModel>("SELECT * FROM user_management.users;")
        .fetch_all(pg_pool)
        .await
        .expect("Could not get all users");

    let all_posts = sqlx::query_as::<_, ContentModel>("SELECT * FROM post_management.posts;")
        .fetch_all(pg_pool)
        .await
        .expect("Could not get all posts");

    (u, v)
}
