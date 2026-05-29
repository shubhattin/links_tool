//! OAuth environment configuration.

use std::sync::OnceLock;
use std::time::Duration;

static HTTP: OnceLock<reqwest::Client> = OnceLock::new();

pub fn http_client() -> &'static reqwest::Client {
    HTTP.get_or_init(|| {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
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

fn parse_http_url(env_key: &str, raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    let parsed =
        reqwest::Url::parse(trimmed).map_err(|e| format!("{env_key} must be a valid URL: {e}"))?;
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(format!("{env_key} must use http or https (got {scheme})"));
        }
    }
    Ok(trimmed.trim_end_matches('/').to_string())
}

pub fn load_oauth_env() -> Result<OAuthEnv, String> {
    let auth_base_raw = std::env::var("AUTH_BASE_URL")
        .or_else(|_| std::env::var("FRONTEND_URL"))
        .map_err(|_| {
            "AUTH_BASE_URL or FRONTEND_URL must be set for OAuth callback URLs".to_string()
        })?;
    let auth_base_url = parse_http_url("AUTH_BASE_URL", &auth_base_raw)?;
    let frontend_url = match std::env::var("FRONTEND_URL") {
        Ok(v) => parse_http_url("FRONTEND_URL", &v)?,
        Err(_) => auth_base_url.clone(),
    };

    Ok(OAuthEnv {
        auth_base_url,
        frontend_url,
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
