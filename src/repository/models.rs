use diesel::prelude::*;
use diesel::sql_types::Timestamp;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::repository::schema::tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub timezone: String,
}

use crate::repository::schema::tasks;

#[derive(Insertable)]
#[diesel(table_name = tasks)]
pub struct NewTask<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub(crate) timezone: &'a str,
}
