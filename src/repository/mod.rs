pub mod models;
pub mod schema;

use std::env;
use diesel::{Connection, PgConnection, RunQueryDsl, SelectableHelper};
use dotenv::dotenv;
use crate::repository::models::{NewTask, Task};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url:?}"))
}

pub fn create_post(conn: &mut PgConnection, name: &str) -> Task {
    use schema::tasks;

    let new_post = NewTask { name, description: None, timezone: "test" };

    diesel::insert_into(tasks::table)
        .values(&new_post)
        .returning(Task::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}