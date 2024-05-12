use argon2::password_hash::{rand_core::OsRng, SaltString};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::hash_scheme::Hasher;

pub struct HmacV01;

type HmacSha256 = Hmac<Sha256>;

impl Hasher for HmacV01 {
    fn hash(
        &self,
        password: &str,
    ) -> crate::error::HashResult<(crate::hash_scheme::Hash, crate::hash_scheme::Salt)> {
        let salt = SaltString::generate(&mut OsRng);
        let mac = HmacSha256::new_from_slice(salt.as_str().as_bytes())?;
        let res = mac.finalize();
        // let str = String::from_utf8_lossy(res..into_bytes().)
        // Ok()
        todo!()
    }
    fn verify(
        &self,
        password: &str,
        salt_str: &crate::hash_scheme::Salt,
        hash_str: &crate::hash_scheme::Hash,
    ) -> crate::error::HashResult<()> {
        todo!()
    }
    fn hash_with_salt(
        &self,
        password: &str,
        salt: &str,
    ) -> crate::error::HashResult<crate::hash_scheme::Hash> {
        todo!()
    }
}
