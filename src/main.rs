use aws_sdk_s3::config::Credentials;
use dotenvy::dotenv;
use itertools::Itertools;
use models::{
    content_model::ContentModel, interactions_matrix_model::build_model, user_model::UserModel,
};
use routes::AppState;
use services::tensor::TensorAppState;
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

    let k_features = 10;
    let (u_embeddings, v_embeddings, a) = load_models(&pool, k_features).await;

    let tensor_app_state = TensorAppState {
        user_embeddings: u_embeddings,
        post_embeddings: v_embeddings,
        interactions: a,
    };

    let app_state = AppState {
        pool,
        s3_client,
        models: Arc::new(tensor_app_state),
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

/// -> (u, v, a)
async fn load_models(pg_pool: &Pool<Postgres>, k: u64) -> (Tensor<f32>, Tensor<f32>, Tensor<f32>) {
    let join_query = build_model(pg_pool)
        .await
        .expect("Could not query for interactions matrix model");

    println!("{:?}", join_query);

    let u_count = join_query
        .iter()
        .unique_by(|q| q.user_id)
        .collect::<Vec<_>>()
        .len() as u64;
    let v_count = join_query
        .iter()
        .unique_by(|q| q.post_id)
        .collect::<Vec<_>>()
        .len() as u64;

    println!("{}-{}", u_count, v_count);

    let id_start = 1000;
    let u = Tensor::new(&[u_count, k]);
    let v = Tensor::new(&[v_count, k]);
    let mut a = Tensor::<f32>::new(&[u_count, v_count, 2]);

    // fills the tensor
    for i in 0..join_query.len() {
        let u_index = (join_query[i].user_id - id_start) as u64;
        let v_index = (join_query[i].post_id - id_start) as u64;
        a.set(&[u_index, v_index, 0], join_query[i].is_liked as f32);
        a.set(&[u_index, v_index, 1], join_query[i].is_following as f32);
    }

    let o = a * v;

    println!("{:?}", a);

    (u, v, a)
}
