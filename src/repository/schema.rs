// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "valid_events"))]
    pub struct ValidEvents;
}

diesel::table! {
    actions (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        task_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ValidEvents;

    task_events (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Varchar>,
        event_type -> ValidEvents,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        task_id -> Int4,
    }
}

diesel::table! {
        tasks (id) {
        id -> Int4,
        name -> Varchar,
        description -> Nullable<Varchar>,
        timezone -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(actions -> tasks (task_id));
diesel::joinable!(task_events -> tasks (task_id));

diesel::allow_tables_to_appear_in_same_query!(
    actions,
    task_events,
    tasks,
);
