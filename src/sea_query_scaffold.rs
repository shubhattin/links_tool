#![allow(dead_code)]
//! Skeleton SeaQuery snippets for upcoming INSERT / UPDATE helpers.
//! Reads continue to use SeaORM entities in [`crate::entities`].

use crate::entities::links;
use sea_orm::EntityName;
use sea_query::{PostgresQueryBuilder, Query};

/// Example SELECT built with SeaQuery (not wired into the HTTP handlers).
///
/// Filtering is folded in once INSERT/UPSERT flows exist; wiring `and_where`
/// ergonomics differs between SeaQuery minor versions—keep template minimal until then.
pub fn example_select_link_ids() -> String {
    Query::select()
        .column(links::Column::Id)
        .column(links::Column::PrefixZeros)
        .from(links::Entity.table_ref())
        .to_owned()
        .to_string(PostgresQueryBuilder)
}
