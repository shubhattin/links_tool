//! Find or create user + provider account from OAuth profile.

use crate::auth::DEFAULT_USER_ROLE;
use crate::entities::{account, user};
use crate::state::AppState;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};

pub const GOOGLE_PROVIDER: &str = "google";
pub const GITHUB_PROVIDER: &str = "github";

#[derive(Debug)]
pub struct OAuthProfile {
    pub provider_id: &'static str,
    pub account_id: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub name: String,
    pub image: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub access_token_expires_at: Option<DateTime<chrono::FixedOffset>>,
    pub scope: Option<String>,
}

#[derive(Debug)]
pub enum LinkError {
    #[allow(unused)]
    Db(sea_orm::DbErr),
    Banned,
    EmailRequired,
    EmailConflict,
}

impl From<sea_orm::DbErr> for LinkError {
    fn from(e: sea_orm::DbErr) -> Self {
        LinkError::Db(e)
    }
}

pub async fn find_or_link_user(
    state: &AppState,
    profile: OAuthProfile,
) -> Result<user::Model, LinkError> {
    if let Some(existing) = account::Entity::find()
        .filter(account::Column::ProviderId.eq(profile.provider_id))
        .filter(account::Column::AccountId.eq(&profile.account_id))
        .one(&state.db)
        .await?
    {
        let user = user::Entity::find_by_id(existing.user_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| LinkError::Db(sea_orm::DbErr::Custom("user missing".into())))?;
        if user.banned {
            return Err(LinkError::Banned);
        }
        update_account_tokens(&state.db, existing.id, &profile).await?;
        return Ok(user);
    }

    let email = profile.email.as_ref().and_then(|e| normalize_email(e));

    if profile.email_verified
        && let Some(ref email) = email
        && let Some(existing_user) = user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&state.db)
            .await?
    {
        if existing_user.banned {
            return Err(LinkError::Banned);
        }
        let already_linked = account::Entity::find()
            .filter(account::Column::UserId.eq(existing_user.id.clone()))
            .filter(account::Column::ProviderId.eq(profile.provider_id))
            .one(&state.db)
            .await?;
        if already_linked.is_some() {
            return Err(LinkError::EmailConflict);
        }
        insert_account(&state.db, &existing_user.id, &profile).await?;
        return Ok(existing_user);
    }

    if let Some(ref email) = email
        && user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&state.db)
            .await?
            .is_some()
    {
        return Err(LinkError::EmailConflict);
    }

    let Some(email) = email else {
        return Err(LinkError::EmailRequired);
    };

    let user_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    let txn = state.db.begin().await?;

    let user_model = user::ActiveModel {
        id: Set(user_id.clone()),
        name: Set(truncate_name(&profile.name)),
        email: Set(email),
        email_verified: Set(profile.email_verified),
        image: Set(profile.image.clone()),
        role: Set(DEFAULT_USER_ROLE.to_string()),
        banned: Set(false),
        ban_reason: Set(None),
        ban_expires: Set(None),
        username: Set(None),
        display_username: Set(None),
        is_maintainer: Set(false),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    user_model.insert(&txn).await?;

    insert_account_in_txn(&txn, &user_id, &profile).await?;
    txn.commit().await?;

    user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| LinkError::Db(sea_orm::DbErr::Custom("user missing after insert".into())))
}

async fn update_account_tokens(
    db: &crate::db::DbPool,
    account_row_id: String,
    profile: &OAuthProfile,
) -> Result<(), LinkError> {
    let now = Utc::now();
    let mut active: account::ActiveModel = account::Entity::find_by_id(account_row_id)
        .one(db)
        .await?
        .ok_or_else(|| LinkError::Db(sea_orm::DbErr::Custom("account missing".into())))?
        .into();
    active.access_token = Set(profile.access_token.clone());
    active.refresh_token = Set(profile.refresh_token.clone());
    active.id_token = Set(profile.id_token.clone());
    active.access_token_expires_at = Set(profile.access_token_expires_at);
    active.scope = Set(profile.scope.clone());
    active.updated_at = Set(now.into());
    active.update(db).await?;
    Ok(())
}

async fn insert_account(
    db: &crate::db::DbPool,
    user_id: &str,
    profile: &OAuthProfile,
) -> Result<(), LinkError> {
    insert_account_in_txn(db, user_id, profile).await
}

async fn insert_account_in_txn<C>(
    db: &C,
    user_id: &str,
    profile: &OAuthProfile,
) -> Result<(), LinkError>
where
    C: sea_orm::ConnectionTrait,
{
    let now = Utc::now();
    let account_model = account::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        account_id: Set(profile.account_id.clone()),
        provider_id: Set(profile.provider_id.to_string()),
        user_id: Set(user_id.to_string()),
        access_token: Set(profile.access_token.clone()),
        refresh_token: Set(profile.refresh_token.clone()),
        id_token: Set(profile.id_token.clone()),
        access_token_expires_at: Set(profile.access_token_expires_at),
        refresh_token_expires_at: Set(None),
        scope: Set(profile.scope.clone()),
        password: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    account_model.insert(db).await?;
    Ok(())
}

fn normalize_email(email: &str) -> Option<String> {
    let email = email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') || email.len() > 320 {
        return None;
    }
    Some(email)
}

fn truncate_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "User".to_string();
    }
    trimmed.chars().take(255).collect()
}
