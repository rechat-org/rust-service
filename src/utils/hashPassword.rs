use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password_and_salt(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, &hashed_password)
}
