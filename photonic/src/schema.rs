// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "medium_item_type_enum"))]
    pub struct MediumItemTypeEnum;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "medium_type_enum"))]
    pub struct MediumTypeEnum;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "store_location_enum"))]
    pub struct StoreLocationEnum;
}

diesel::table! {
    albums (id) {
        id -> Uuid,
        owner_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        title_medium -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediumTypeEnum;

    media (id) {
        id -> Uuid,
        owner_id -> Uuid,
        medium_type -> MediumTypeEnum,
        album_id -> Nullable<Uuid>,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    media_tags (medium_id, tag_id) {
        medium_id -> Uuid,
        tag_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MediumItemTypeEnum;
    use super::sql_types::StoreLocationEnum;

    medium_items (id) {
        id -> Uuid,
        medium_id -> Uuid,
        medium_item_type -> MediumItemTypeEnum,
        #[max_length = 100]
        mime -> Varchar,
        #[max_length = 255]
        filename -> Varchar,
        #[max_length = 1024]
        path -> Varchar,
        size -> Int8,
        location -> StoreLocationEnum,
        priority -> Int4,
        timezone -> Int4,
        taken_at -> Timestamptz,
        last_saved -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
        width -> Int4,
        height -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::StoreLocationEnum;

    sidecars (id) {
        id -> Uuid,
        medium_id -> Uuid,
        #[max_length = 100]
        mime -> Varchar,
        #[max_length = 255]
        filename -> Varchar,
        #[max_length = 1024]
        path -> Varchar,
        size -> Int8,
        location -> StoreLocationEnum,
        priority -> Int4,
        last_saved -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    tags (id) {
        id -> Uuid,
        #[max_length = 100]
        title -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        username -> Nullable<Varchar>,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        #[max_length = 255]
        given_name -> Nullable<Varchar>,
        quota -> Int8,
        quota_used -> Int8,
    }
}

diesel::joinable!(albums -> users (owner_id));
diesel::joinable!(media -> users (owner_id));
diesel::joinable!(media_tags -> media (medium_id));
diesel::joinable!(media_tags -> tags (tag_id));
diesel::joinable!(medium_items -> media (medium_id));
diesel::joinable!(sidecars -> media (medium_id));

diesel::allow_tables_to_appear_in_same_query!(
    albums,
    media,
    media_tags,
    medium_items,
    sidecars,
    tags,
    users,
);
