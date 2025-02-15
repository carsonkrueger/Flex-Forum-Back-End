use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher},
    Argon2, PasswordHash, PasswordVerifier,
};
use lib_macros::{iterator_def, iterator_iden_def};
use sqlx::prelude::FromRow;

use crate::{
    model::{
        base,
        schema::FlexForumDbConnection,
        schemas::user_management::users::{username_or_email_exists, Users, UsersIden},
    },
    route::error::{RouteError, RouteResult},
};

#[derive(FromRow, Debug)]
#[iterator_iden_def(UsersIden)]
#[iterator_def]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub pwd_hash: String,
}

pub async fn create_user<'e>(
    username: &str,
    email: &str,
    password: &str,
    pool: &mut FlexForumDbConnection,
) -> RouteResult<Users> {
    let taken_str = username_or_email_exists(&username, &email, &mut *pool).await?;
    if let Some(taken) = taken_str {
        return Err(RouteError::AlreadyTaken(taken));
    }

    let argon2 = Argon2::default();
    let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;

    let create_user = CreateUser {
        email: email.to_string(),
        username: username.to_string(),
        pwd_hash: hash.to_string(),
    };

    let user = base::insert_returning::<Users, CreateUser>(create_user, pool).await?;

    Ok(user)
}

pub fn verify_user(user: &Users, password: &str) -> RouteResult<()> {
    let hash = PasswordHash::new(&user.pwd_hash)?;
    let argon2 = Argon2::default();
    argon2.verify_password(password.as_bytes(), &hash)?;
    Ok(())
}
