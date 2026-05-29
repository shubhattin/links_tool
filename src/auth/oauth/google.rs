//! Google sign-in via OpenID Connect (authorization code + PKCE).

use crate::auth::oauth::config::{OAuthEnv, http_client};
use crate::auth::oauth::link::{GOOGLE_PROVIDER, OAuthProfile};
use crate::auth::oauth::state_cookie::OAuthPending;
use oauth2::PkceCodeVerifier;
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse,
};

pub struct GoogleStart {
    pub authorize_url: String,
    pub pending: OAuthPending,
}

pub async fn start_google_async(env: &OAuthEnv) -> Result<GoogleStart, String> {
    let metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new("https://accounts.google.com".to_string()).map_err(|e| e.to_string())?,
        http_client(),
    )
    .await
    .map_err(|e| format!("google discovery failed: {e}"))?;

    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(env.google_client_id.clone()),
        Some(ClientSecret::new(env.google_client_secret.clone())),
    )
    .set_redirect_uri(RedirectUrl::new(env.google_callback_url()).map_err(|e| e.to_string())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let pending = crate::auth::oauth::state_cookie::new_pending(
        GOOGLE_PROVIDER,
        csrf_token.secret().clone(),
        pkce_verifier.secret().clone(),
        Some(nonce.secret().clone()),
    );

    Ok(GoogleStart {
        authorize_url: auth_url.to_string(),
        pending,
    })
}

pub async fn finish_google(
    env: &OAuthEnv,
    pending: &OAuthPending,
    code: &str,
    returned_state: &str,
) -> Result<OAuthProfile, String> {
    if pending.provider != GOOGLE_PROVIDER {
        return Err("oauth provider mismatch".into());
    }
    if pending.state != returned_state {
        return Err("invalid oauth state".into());
    }

    let metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new("https://accounts.google.com".to_string()).map_err(|e| e.to_string())?,
        http_client(),
    )
    .await
    .map_err(|e| format!("google discovery failed: {e}"))?;

    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(env.google_client_id.clone()),
        Some(ClientSecret::new(env.google_client_secret.clone())),
    )
    .set_redirect_uri(RedirectUrl::new(env.google_callback_url()).map_err(|e| e.to_string())?);

    let pkce_verifier = PkceCodeVerifier::new(pending.pkce_verifier.clone());
    let nonce = Nonce::new(
        pending
            .nonce
            .clone()
            .ok_or_else(|| "missing oauth nonce".to_string())?,
    );

    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .map_err(|e| e.to_string())?
        .set_pkce_verifier(pkce_verifier)
        .request_async(http_client())
        .await
        .map_err(|e| format!("google token exchange failed: {e}"))?;

    let id_token = token_response
        .id_token()
        .ok_or_else(|| "google did not return an id token".to_string())?;
    let id_token_verifier = client.id_token_verifier();
    let claims = id_token
        .claims(&id_token_verifier, &nonce)
        .map_err(|e| format!("invalid google id token: {e}"))?;

    let sub = claims.subject().to_string();
    let email = claims.email().map(|e| e.to_string());
    let email_verified = claims.email_verified().unwrap_or(false);
    let name = claims
        .name()
        .and_then(|n| n.get(None))
        .map(|n| n.to_string())
        .unwrap_or_else(|| email.clone().unwrap_or_else(|| "User".to_string()));
    let picture = claims
        .picture()
        .and_then(|p| p.get(None))
        .map(|p| p.to_string());

    let expires_at = token_response
        .expires_in()
        .map(|d| chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64));

    Ok(OAuthProfile {
        provider_id: GOOGLE_PROVIDER,
        account_id: sub,
        email,
        email_verified,
        name,
        image: picture,
        access_token: Some(token_response.access_token().secret().clone()),
        refresh_token: token_response.refresh_token().map(|t| t.secret().clone()),
        id_token: Some(id_token.to_string()),
        access_token_expires_at: expires_at.map(|dt| dt.fixed_offset()),
        scope: token_response.scopes().map(|scopes| {
            scopes
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        }),
    })
}
