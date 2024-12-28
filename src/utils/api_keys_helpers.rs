use sha2::{Digest, Sha256};

pub fn generate_api_key_prefix(api_key: &str) -> String {
    let prefix = if api_key.len() >= 8 {
        &api_key[..8]
    } else {
        api_key
    };

    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hex::encode(hasher.finalize())
}
