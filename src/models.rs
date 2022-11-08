use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
pub struct User {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
}

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "posts")] // singular 'user' is a keyword..
pub struct Post {
    pub name: String,
    pub icon: String,
    pub content: String,
    pub media: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}
