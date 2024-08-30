use super::jwt::JWT;

#[derive(Clone, Debug)]
pub struct Ctx {
    jwt: JWT,
}

impl Ctx {
    pub fn new(jwt: JWT) -> Self {
        Self { jwt }
    }
    pub fn jwt(&self) -> &JWT {
        &self.jwt
    }
}
