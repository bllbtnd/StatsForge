use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub github_token: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let github_token = env::var("GITHUB_TOKEN")
            .map_err(|_| "GITHUB_TOKEN environment variable is required".to_string())?;

        if github_token.trim().is_empty() {
            return Err("GITHUB_TOKEN must not be empty".to_string());
        }

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| "PORT must be a valid port number (0–65535)".to_string())?;

        Ok(Config {
            github_token,
            port,
        })
    }
}
