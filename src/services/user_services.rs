use sqlx::PgPool;

pub async fn email_exists(email: &str, pool: &PgPool) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("SELECT email FROM user_management.users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await;
    match result {
        Err(e) => Err(e),
        Ok(r) => Ok(r.is_some()),
    }
}
