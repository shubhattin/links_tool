//! GitHub OAuth2 sign-in (authorization code + PKCE).

use crate::auth::oauth::config::{OAuthEnv, http_client};
use crate::auth::oauth::link::{GITHUB_PROVIDER, OAuthProfile};
use crate::auth::oauth::state_cookie::OAuthPending;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

pub struct GithubStart {
    pub authorize_url: String,
    pub pending: OAuthPending,
}

pub async fn start_github_async(env: &OAuthEnv) -> Result<GithubStart, String> {
    let client = BasicClient::new(ClientId::new(env.github_client_id.clone()))
        .set_client_secret(ClientSecret::new(env.github_client_secret.clone()))
        .set_auth_uri(AuthUrl::new(GITHUB_AUTH_URL.to_string()).map_err(|e| e.to_string())?)
        .set_token_uri(TokenUrl::new(GITHUB_TOKEN_URL.to_string()).map_err(|e| e.to_string())?)
        .set_redirect_uri(RedirectUrl::new(env.github_callback_url()).map_err(|e| e.to_string())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let pending = crate::auth::oauth::state_cookie::new_pending(
        GITHUB_PROVIDER,
        csrf_token.secret().clone(),
        pkce_verifier.secret().clone(),
        None,
    );

    Ok(GithubStart {
        authorize_url: auth_url.to_string(),
        pending,
    })
}

pub async fn finish_github(
    env: &OAuthEnv,
    pending: &OAuthPending,
    code: &str,
    returned_state: &str,
) -> Result<OAuthProfile, String> {
    if pending.provider != GITHUB_PROVIDER {
        return Err("oauth provider mismatch".into());
    }
    if pending.state != returned_state {
        return Err("invalid oauth state".into());
    }

    let client = BasicClient::new(ClientId::new(env.github_client_id.clone()))
        .set_client_secret(ClientSecret::new(env.github_client_secret.clone()))
        .set_auth_uri(AuthUrl::new(GITHUB_AUTH_URL.to_string()).map_err(|e| e.to_string())?)
        .set_token_uri(TokenUrl::new(GITHUB_TOKEN_URL.to_string()).map_err(|e| e.to_string())?)
        .set_redirect_uri(RedirectUrl::new(env.github_callback_url()).map_err(|e| e.to_string())?);

    let pkce_verifier = PkceCodeVerifier::new(pending.pkce_verifier.clone());

    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(http_client())
        .await
        .map_err(|e| format!("github token exchange failed: {e}"))?;

    let access_token = token_response.access_token().secret().clone();
    let gh_user = fetch_github_user(&access_token).await?;
    let email = resolve_github_email(&access_token, &gh_user).await?;

    let name = gh_user
        .name
        .clone()
        .or(gh_user.login.clone())
        .unwrap_or_else(|| "User".to_string());

    Ok(OAuthProfile {
        provider_id: GITHUB_PROVIDER,
        account_id: gh_user.id.to_string(),
        email: Some(email.email),
        email_verified: email.verified,
        name,
        image: gh_user.avatar_url,
        access_token: Some(access_token),
        refresh_token: token_response.refresh_token().map(|t| t.secret().clone()),
        id_token: None,
        access_token_expires_at: token_response.expires_in().map(|d| {
            (chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64)).fixed_offset()
        }),
        scope: token_response.scopes().map(|scopes| {
            scopes
                .iter()
                .flat_map(|comma_separated| comma_separated.split(','))
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        }),
    })
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    id: u64,
    login: Option<String>,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Debug)]
struct ResolvedGithubEmail {
    email: String,
    verified: bool,
}

async fn fetch_github_user(access_token: &str) -> Result<GithubUser, String> {
    let resp = http_client()
        .get("https://api.github.com/user")
        .header("User-Agent", "links-tool")
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("github user request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("github user api returned {}", resp.status()));
    }

    resp.json::<GithubUser>()
        .await
        .map_err(|e| format!("github user parse failed: {e}"))
}

async fn resolve_github_email(
    access_token: &str,
    user: &GithubUser,
) -> Result<ResolvedGithubEmail, String> {
    let resp = http_client()
        .get("https://api.github.com/user/emails")
        .header("User-Agent", "links-tool")
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("github emails request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("github emails api returned {}", resp.status()));
    }

    let emails: Vec<GithubEmail> = resp
        .json()
        .await
        .map_err(|e| format!("github emails parse failed: {e}"))?;

    if let Some(email) = emails
        .iter()
        .find(|e| e.primary && e.verified)
        .or_else(|| emails.iter().find(|e| e.verified))
        .or_else(|| emails.iter().find(|e| e.primary))
    {
        return Ok(ResolvedGithubEmail {
            email: email.email.to_lowercase(),
            verified: email.verified,
        });
    }

    if let Some(email) = user.email.as_ref().filter(|e| !e.is_empty()) {
        return Ok(ResolvedGithubEmail {
            email: email.to_lowercase(),
            verified: false,
        });
    }

    Err("github account has no accessible email".to_string())
}
