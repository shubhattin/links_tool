use crate::auth::REFRESH_TOKEN_TTL_SECS;
use crate::db::DbPool;
use crate::entities::session;
use chrono::{Duration, Utc};
use rand::RngCore;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::Mutex;

/// Minimum interval between expired-session purges per process
const PURGE_MIN_INTERVAL: StdDuration = StdDuration::from_secs(5 * 60 * 60);

struct PurgeThrottle {
    last_purge: Option<Instant>,
}

static PURGE_THROTTLE: OnceLock<Mutex<PurgeThrottle>> = OnceLock::new();

fn purge_throttle() -> &'static Mutex<PurgeThrottle> {
    PURGE_THROTTLE.get_or_init(|| Mutex::new(PurgeThrottle { last_purge: None }))
}

fn hash_token(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    hex::encode(digest)
}

fn generate_raw_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub struct IssuedRefresh {
    pub raw_token: String,
    pub session_id: String,
}

#[derive(Debug)]
pub enum RotateRefreshError {
    SessionAlreadyRotated,
    #[allow(dead_code)]
    Db(sea_orm::DbErr),
}

pub async fn issue_refresh_session(
    db: &DbPool,
    user_id: &str,
) -> Result<IssuedRefresh, sea_orm::DbErr> {
    let raw = generate_raw_token();
    let token_hash = hash_token(&raw);
    let now = Utc::now();
    let expires = now + Duration::seconds(REFRESH_TOKEN_TTL_SECS);
    let session_id = uuid::Uuid::new_v4().to_string();

    let model = session::ActiveModel {
        id: Set(session_id.clone()),
        user_id: Set(user_id.to_string()),
        token_hash: Set(token_hash),
        expires_at: Set(expires.into()),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    model.insert(db).await?;

    Ok(IssuedRefresh {
        raw_token: raw,
        session_id,
    })
}

pub async fn rotate_refresh_session(
    db: &DbPool,
    old_session_id: &str,
    user_id: &str,
) -> Result<IssuedRefresh, RotateRefreshError> {
    let delete_result = session::Entity::delete_by_id(old_session_id)
        .exec(db)
        .await
        .map_err(RotateRefreshError::Db)?;
    if delete_result.rows_affected == 0 {
        return Err(RotateRefreshError::SessionAlreadyRotated);
    }
    issue_refresh_session(db, user_id)
        .await
        .map_err(RotateRefreshError::Db)
}

pub async fn find_valid_session_by_token(
    db: &DbPool,
    raw_token: &str,
) -> Result<Option<session::Model>, sea_orm::DbErr> {
    let token_hash = hash_token(raw_token);
    let row = session::Entity::find()
        .filter(session::Column::TokenHash.eq(token_hash))
        .one(db)
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let now: sea_orm::prelude::DateTimeWithTimeZone = Utc::now().into();
    if row.expires_at < now {
        session::Entity::delete_by_id(row.id.clone())
            .exec(db)
            .await?;
        return Ok(None);
    }
    Ok(Some(row))
}

pub async fn revoke_session(db: &DbPool, session_id: &str) -> Result<(), sea_orm::DbErr> {
    session::Entity::delete_by_id(session_id).exec(db).await?;
    Ok(())
}

/// Delete expired sessions when due — at most once per [`PURGE_MIN_INTERVAL`] per process.
pub async fn maybe_purge_expired_sessions(db: &DbPool) -> Result<u64, sea_orm::DbErr> {
    let mut throttle = purge_throttle().lock().await;
    if let Some(last) = throttle.last_purge
        && last.elapsed() < PURGE_MIN_INTERVAL
    {
        return Ok(0);
    }
    let deleted = purge_expired_sessions(db).await?;
    throttle.last_purge = Some(Instant::now());
    Ok(deleted)
}

async fn purge_expired_sessions(db: &DbPool) -> Result<u64, sea_orm::DbErr> {
    let now: sea_orm::prelude::DateTimeWithTimeZone = Utc::now().into();
    let result = session::Entity::delete_many()
        .filter(session::Column::ExpiresAt.lt(now))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

#[cfg(test)]
mod purge_throttle_tests {
    use super::*;

    #[test]
    fn purge_interval_is_five_hours() {
        assert_eq!(PURGE_MIN_INTERVAL, StdDuration::from_secs(5 * 60 * 60));
    }

    fn purge_due(last: Option<Instant>, now: Instant) -> bool {
        match last {
            None => true,
            Some(last) => now.duration_since(last) >= PURGE_MIN_INTERVAL,
        }
    }

    #[test]
    fn first_purge_always_due() {
        assert!(purge_due(None, Instant::now()));
    }

    #[test]
    fn purge_skipped_within_interval() {
        let now = Instant::now();
        let last = now - StdDuration::from_secs(60);
        assert!(!purge_due(Some(last), now));
    }

    #[test]
    fn purge_runs_after_interval() {
        let now = Instant::now();
        let last = now - PURGE_MIN_INTERVAL - StdDuration::from_secs(1);
        assert!(purge_due(Some(last), now));
    }
}
