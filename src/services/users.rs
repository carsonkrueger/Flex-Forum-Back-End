use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher},
    Argon2, PasswordHash, PasswordVerifier,
};
use lib_macros::{iterator_def, iterator_iden_def};
use sqlx::prelude::FromRow;

use crate::{
    model::{
        base::BaseModelTrait,
        schema::FlexForumDbConnection,
        schemas::user_management::users::{Users, UsersIden, UsersModelTrait},
    },
    route::{
        error::{RouteError, RouteResult},
        public_routes::auth::SignUpModel,
    },
};

#[derive(FromRow, Debug)]
#[iterator_iden_def(UsersIden)]
#[iterator_def]
pub struct CreateUser {
    pub email: String,
    pub username: String,
    pub pwd_hash: String,
}

pub trait UsersServiceTrait {
    async fn create_user<BM: BaseModelTrait>(
        sign_up: &SignUpModel,
        pool: &mut FlexForumDbConnection,
    ) -> RouteResult<Users>;
    async fn verify_user<BM: BaseModelTrait>(
        username: &str,
        password: &str,
        pool: &mut FlexForumDbConnection,
    ) -> RouteResult<Users>;
}

pub struct UsersService;

impl UsersServiceTrait for UsersService {
    async fn create_user<BM: BaseModelTrait>(
        sign_up: &SignUpModel,
        pool: &mut FlexForumDbConnection,
    ) -> RouteResult<Users> {
        let taken_str =
            Users::username_or_email_exists(&sign_up.username, &sign_up.email, &mut *pool).await?;
        if let Some(taken) = taken_str {
            return Err(RouteError::AlreadyTaken(taken));
        }

        let argon2 = Argon2::default();
        let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
        let hash = argon2.hash_password(sign_up.password.as_bytes(), &salt)?;

        let create_user = CreateUser {
            email: sign_up.email.to_string(),
            username: sign_up.username.to_string(),
            pwd_hash: hash.to_string(),
        };

        let user = BM::insert_returning::<Users, CreateUser>(create_user, pool).await?;

        Ok(user)
    }
    async fn verify_user<BM: BaseModelTrait>(
        username: &str,
        password: &str,
        pool: &mut FlexForumDbConnection,
    ) -> RouteResult<Users> {
        let user = BM::get_one_with::<Users, Users>(UsersIden::Username, username, pool)
            .await?
            .ok_or(RouteError::InvalidAuth)?;

        let hash = PasswordHash::new(&user.pwd_hash)?;
        let argon2 = Argon2::default();
        argon2.verify_password(password.as_bytes(), &hash)?;
        Ok(user)
    }
}
