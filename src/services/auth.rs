use crate::{
    libs::jwt::JWT,
    routes::{RouteError, RouterResult},
};

pub fn check_username(username: &str, jwt: &JWT) -> RouterResult<()> {
    if username != jwt.username() {
        return Err(RouteError::Unauthorized);
    }
    Ok(())
}
