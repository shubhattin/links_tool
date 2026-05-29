//! OAuth environment configuration.

use std::sync::OnceLock;

static HTTP: OnceLock<reqwest::Client> = OnceLock::new();

pub fn http_client() -> &'static reqwest::Client {
    HTTP.get_or_init(|| {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("reqwest client")
    })
}

#[derive(Clone, Debug)]
pub struct OAuthEnv {
    pub auth_base_url: String,
    pub frontend_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
}

impl OAuthEnv {
    pub fn google_callback_url(&self) -> String {
        format!(
            "{}/api/auth/callback/google",
            self.auth_base_url.trim_end_matches('/')
        )
    }

    pub fn github_callback_url(&self) -> String {
        format!(
            "{}/api/auth/callback/github",
            self.auth_base_url.trim_end_matches('/')
        )
    }
}

pub fn load_oauth_env() -> Result<OAuthEnv, String> {
    let auth_base_url = std::env::var("AUTH_BASE_URL")
        .or_else(|_| std::env::var("FRONTEND_URL"))
        .map(|v| v.trim().to_string())
        .map_err(|_| {
            "AUTH_BASE_URL or FRONTEND_URL must be set for OAuth callback URLs".to_string()
        })?;
    let frontend_url = std::env::var("FRONTEND_URL")
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|_| auth_base_url.clone());

    Ok(OAuthEnv {
        auth_base_url: auth_base_url.trim_end_matches('/').to_string(),
        frontend_url: frontend_url.trim_end_matches('/').to_string(),
        google_client_id: required_env("GOOGLE_CLIENT_ID")?,
        google_client_secret: required_env("GOOGLE_CLIENT_SECRET")?,
        github_client_id: required_env("GITHUB_CLIENT_ID")?,
        github_client_secret: required_env("GITHUB_CLIENT_SECRET")?,
    })
}

fn required_env(key: &str) -> Result<String, String> {
    let value = std::env::var(key)
        .map_err(|_| format!("{key} must be set: missing environment variable"))?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{key} must be set: environment variable is empty"));
    }
    Ok(trimmed.to_string())
}
