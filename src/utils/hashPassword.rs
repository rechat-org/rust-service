use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password_and_salt(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}
