use crate::auth::ACCESS_TOKEN_TTL_SECS;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn issue_access_token(
    secret: &str,
    user_id: &str,
) -> Result<(String, i64), jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::seconds(ACCESS_TOKEN_TTL_SECS);
    let claims = AccessClaims {
        sub: user_id.to_string(),
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok((token, ACCESS_TOKEN_TTL_SECS))
}

pub fn verify_access_token(
    secret: &str,
    token: &str,
) -> Result<AccessClaims, jsonwebtoken::errors::Error> {
    let data = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_and_verify() {
        let secret = "test-secret-key-at-least-32-bytes!!";
        let (token, ttl) = issue_access_token(secret, "u1").unwrap();
        assert_eq!(ttl, ACCESS_TOKEN_TTL_SECS);
        let claims = verify_access_token(secret, &token).unwrap();
        assert_eq!(claims.sub, "u1");
    }
}
