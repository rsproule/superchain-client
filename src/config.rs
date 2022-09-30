use std::env;

use dotenv::dotenv;

pub struct Config {
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        let username = env::var("SC_USERNAME").expect("SC_USERNAME environment variable");
        let password = env::var("SC_PASSWORD").expect("SC_PASSWORD environment variable");
        Config { username, password }
    }

    pub fn get_basic_authorization_value(&self) -> String {
        let encoded = base64::encode(format!("{}:{}", self.username, self.password));
        format!("Basic {encoded}")
    }
}
