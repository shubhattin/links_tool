use crate::db::DbPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: Arc<str>,
}
