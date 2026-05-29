use crate::auth::REFRESH_TOKEN_TTL_SECS;
use crate::db::DbPool;
use crate::entities::session;
use chrono::{Duration, Utc};
use rand::RngCore;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};

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
