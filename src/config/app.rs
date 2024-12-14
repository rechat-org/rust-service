use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: String,
}

impl AppConfig {
    pub fn new() -> Self {
        let host = env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string());

        Self { host, port }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}