//! Database schema (mirrors `app/src/db/schema.ts` Drizzle definitions).

diesel::table! {
    links (id) {
        id -> Varchar,
        enabled -> Bool,
        link -> Text,
        prefix_zeros -> Integer,
        name -> Nullable<Varchar>,
    }
}

diesel::table! {
    others (key) {
        key -> Varchar,
        value -> Text,
    }
}
