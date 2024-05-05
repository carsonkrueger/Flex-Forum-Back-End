use axum::http::StatusCode;
use sqlx::PgPool;

pub async fn email_exists(email: &str, pool: &PgPool) -> Result<bool, StatusCode> {
    let result = sqlx::query("SELECT email from user_management.users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await;
    match result {
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(r) => Ok(r.is_some()),
    }
}
