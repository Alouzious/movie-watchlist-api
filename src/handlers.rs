use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db;
use crate::models::{CreateMovie, UpdateMovie, RegisterUser, LoginUser, Claims};

fn extract_user_id(headers: &HeaderMap) -> Option<Uuid> {
    let auth = headers.get("Authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    let secret = env::var("JWT_SECRET").ok()?;
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).ok()?;
    Uuid::parse_str(&decoded.claims.user_id).ok()
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(body): Json<RegisterUser>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let existing = db::find_user_by_username(&pool, &body.username).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing.is_some() {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "message": "Username already exists"
        })));
    }

    let password_hash = hash(&body.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = db::create_user(&pool, &body.username, &password_hash).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "User registered successfully",
        "data": { "id": user.id, "username": user.username }
    })))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(body): Json<LoginUser>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = db::find_user_by_username(&pool, &body.username).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = match user {
        Some(u) => u,
        None => return Ok(Json(serde_json::json!({
            "status": "error",
            "message": "Invalid username or password"
        }))),
    };

    let valid = verify(&body.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !valid {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "message": "Invalid username or password"
        })));
    }

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 86400;

    let claims = Claims {
        sub: user.username.clone(),
        user_id: user.id.to_string(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "token": token,
        "username": user.username
    })))
}

pub async fn get_all_movies(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = extract_user_id(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    match db::get_all_movies(&pool, user_id).await {
        Ok(movies) => Ok(Json(serde_json::json!({
            "status": "success",
            "count": movies.len(),
            "data": movies
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_movie_by_id(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = extract_user_id(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    match db::get_movie_by_id(&pool, id, user_id).await {
        Ok(Some(movie)) => Ok(Json(serde_json::json!({
            "status": "success",
            "data": movie
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_movie(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(body): Json<CreateMovie>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = extract_user_id(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    match db::create_movie(&pool, body, user_id).await {
        Ok(movie) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Movie created successfully",
            "data": movie
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_movie(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMovie>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = extract_user_id(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    match db::update_movie(&pool, id, user_id, body).await {
        Ok(Some(movie)) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Movie updated successfully",
            "data": movie
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_movie(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = extract_user_id(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    match db::delete_movie(&pool, id, user_id).await {
        Ok(true) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Movie deleted successfully"
        }))),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}