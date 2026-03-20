mod models;
mod db;
mod handlers;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use std::env;
use tower_http::cors::{CorsLayer, Any};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    db::create_tables(&pool)
        .await
        .expect("Failed to create tables");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/auth/register", post(handlers::register))
        .route("/auth/login", post(handlers::login))
        .route("/movies", get(handlers::get_all_movies))
        .route("/movies", post(handlers::create_movie))
        .route("/movies/:id", get(handlers::get_movie_by_id))
        .route("/movies/:id", put(handlers::update_movie))
        .route("/movies/:id", delete(handlers::delete_movie))
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Server running on http://localhost:8080");

    axum::serve(listener, app).await.unwrap();
}