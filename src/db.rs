use sqlx::{PgPool, Error};
use uuid::Uuid;
use crate::models::{Movie, CreateMovie, UpdateMovie, User};

pub async fn create_tables(pool: &PgPool) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(255) UNIQUE NOT NULL,
            password_hash TEXT NOT NULL
        )"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS movies (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            genre VARCHAR(100) NOT NULL,
            release_year INTEGER NOT NULL,
            poster_url TEXT NOT NULL,
            rating INTEGER CHECK (rating >= 1 AND rating <= 5) NOT NULL,
            status VARCHAR(50) CHECK (status IN ('watched', 'unwatched', 'watching')) NOT NULL
        )"
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn create_user(pool: &PgPool, username: &str, password_hash: &str) -> Result<User, Error> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, username, password_hash) VALUES (gen_random_uuid(), $1, $2) RETURNING *"
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_all_movies(pool: &PgPool, user_id: Uuid) -> Result<Vec<Movie>, Error> {
    let movies = sqlx::query_as::<_, Movie>(
        "SELECT * FROM movies WHERE user_id = $1 ORDER BY title"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(movies)
}

pub async fn get_movie_by_id(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Option<Movie>, Error> {
    let movie = sqlx::query_as::<_, Movie>(
        "SELECT * FROM movies WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(movie)
}

pub async fn create_movie(pool: &PgPool, data: CreateMovie, user_id: Uuid) -> Result<Movie, Error> {
    let movie = sqlx::query_as::<_, Movie>(
        "INSERT INTO movies (id, user_id, title, description, genre, release_year, poster_url, rating, status)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *"
    )
    .bind(user_id)
    .bind(data.title)
    .bind(data.description)
    .bind(data.genre)
    .bind(data.release_year)
    .bind(data.poster_url)
    .bind(data.rating)
    .bind(data.status)
    .fetch_one(pool)
    .await?;
    Ok(movie)
}

pub async fn update_movie(pool: &PgPool, id: Uuid, user_id: Uuid, data: UpdateMovie) -> Result<Option<Movie>, Error> {
    let existing = get_movie_by_id(pool, id, user_id).await?;
    if existing.is_none() {
        return Ok(None);
    }
    let existing = existing.unwrap();

    let movie = sqlx::query_as::<_, Movie>(
        "UPDATE movies
        SET title = $1, description = $2, genre = $3,
            release_year = $4, poster_url = $5, rating = $6, status = $7
        WHERE id = $8 AND user_id = $9
        RETURNING *"
    )
    .bind(data.title.unwrap_or(existing.title))
    .bind(data.description.unwrap_or(existing.description))
    .bind(data.genre.unwrap_or(existing.genre))
    .bind(data.release_year.unwrap_or(existing.release_year))
    .bind(data.poster_url.unwrap_or(existing.poster_url))
    .bind(data.rating.unwrap_or(existing.rating))
    .bind(data.status.unwrap_or(existing.status))
    .bind(id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(Some(movie))
}

pub async fn delete_movie(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool, Error> {
    let result = sqlx::query(
        "DELETE FROM movies WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}