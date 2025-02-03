use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher},
    Argon2, PasswordHash, PasswordVerifier,
};
use sea_query::Expr;
use sqlx::{Pool, Postgres};

use crate::{
    model::{
        base,
        schemas::user_management::users::{username_or_email_exists, Users},
    },
    route::error::{RouteError, RouteResult},
};

pub async fn create_user(
    username: &str,
    email: &str,
    password: &str,
    pool: &Pool<Postgres>,
) -> RouteResult<Users> {
    let taken_str = username_or_email_exists(&username, &email, pool).await?;
    if let Some(taken) = taken_str {
        return Err(RouteError::AlreadyTaken(taken));
    }

    let argon2 = Argon2::default();
    let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;

    let user = base::insert_returning::<Users>(
        [Expr::val(email).into(), Expr::val(&hash.to_string()).into()],
        pool,
    )
    .await?;

    Ok(user)
}

pub fn verify_user(user: &Users, password: &str) -> RouteResult<()> {
    let hash = PasswordHash::new(&user.pwd_hash)?;
    let argon2 = Argon2::default();
    argon2.verify_password(password.as_bytes(), &hash)?;
    Ok(())
}
