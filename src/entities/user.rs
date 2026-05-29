use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub role: String,
    pub banned: bool,
    pub ban_reason: Option<String>,
    pub ban_expires: Option<DateTimeWithTimeZone>,
    pub username: Option<String>,
    pub display_username: Option<String>,
    pub is_maintainer: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::account::Entity")]
    Account,
    #[sea_orm(has_many = "super::session::Entity")]
    Session,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
