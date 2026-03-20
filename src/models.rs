use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Movie {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: String,
    pub genre: String,
    pub release_year: i32,
    pub poster_url: String,
    pub rating: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMovie {
    pub title: String,
    pub description: String,
    pub genre: String,
    pub release_year: i32,
    pub poster_url: String,
    pub rating: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMovie {
    pub title: Option<String>,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub release_year: Option<i32>,
    pub poster_url: Option<String>,
    pub rating: Option<i32>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub exp: usize,
}