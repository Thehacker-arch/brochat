use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub avatar_url: String
}
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct MessageModel {
    pub id: Uuid,
    pub sender: String,
    pub target_username: Option<String>,
    pub message: String,
    pub message_type: String,
    pub timestamp: DateTime<Utc>,
    pub avatar_url: Option<String>,
    pub upload_url: Option<String>
}


#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // subject, e.g. username
    exp: usize,  // expiration time
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct UserList{
    username: String
}

impl AuthenticatedUser {
    pub async  fn from_auth_header(headers: HeaderMap, pool: &PgPool) -> Result<Self, StatusCode> {
        let auth_header = headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = auth_header.trim_start_matches("Bearer ").trim();

        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        
        // let claims = decode_jwt(token).map_err(|_|()).unwrap();

        let userid = Uuid::parse_str(&decoded.claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;
        let user = User::find_by_id(pool, &userid)
        .await.map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(Self {
            id: user.id,
            username: user.username,
            avatar_url: Some(user.avatar_url)
        })
    }
}


impl User {
    pub async fn create(pool: &PgPool, username: &str, password_hash: &str) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(id)
        .bind(username)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }


    pub async fn find_by_username(pool: &sqlx::PgPool, username: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as(
            "SELECT * FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_one(pool)
        .await
    }


    pub async fn find_by_id(pool: &PgPool, user_id: &Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, username, password_hash, avatar_url FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
    }


    pub async fn find_all(pool: &PgPool) -> Result<Vec<UserList>, sqlx::Error> {
        sqlx::query_as::<_, UserList>(
            "SELECT username FROM users"
        )
        .fetch_all(pool)
        .await
    }


    pub async fn insert_pfp(pool: &PgPool, id: Option<Uuid>, avatar_url: String) -> Result<Self, sqlx::Error> {
        let abc = sqlx::query_as::<_, User>(
            "UPDATE users SET avatar_url = $1 WHERE id = $2 RETURNING *"
        )
        .bind(avatar_url)
        .bind(id)
        .fetch_one(pool).await?;

        Ok(abc)
    }

}


impl MessageModel {
    pub async fn save_message(
        pool: &PgPool,
        username: &str,
        message_type: &str,
        message: &str,
        timestamp: &DateTime<Utc>,
        target_username: Option<&str>,
        upload_url: Option<String>
    ) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
    
        if message_type == "dm" && target_username.is_none() {
            panic!("DM message must have a target_username");
        }
    
        sqlx::query!(
            r#"
            INSERT INTO messages (id, sender, target_username, message_type, message, upload_url, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            id,
            username,
            target_username,
            message_type,
            message,
            upload_url,
            timestamp
        )
        .execute(pool)
        .await?;
    
        Ok(())
    }
    

    pub async fn get_public_messages(pool: &PgPool) -> Result<Vec<MessageModel>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MessageModel>(
            r#"
                SELECT
                    messages.id,
                    messages.sender,
                    messages.message,
                    messages.target_username,
                    messages.message_type,
                    messages.timestamp,
                    messages.upload_url,
                    users.avatar_url
                FROM messages
                JOIN users ON messages.sender = users.username
                WHERE messages.message_type = 'chat'
                ORDER BY messages.timestamp ASC
                LIMIT $1;
            "#
        )
        .bind(100)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }


    pub async fn get_dm_messages(pool: &PgPool, current_user: &str, target_user: &str) -> Result<Vec<MessageModel>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MessageModel>(
            r#"
            SELECT
                messages.id,
                messages.sender,
                messages.message,
                messages.target_username,
                messages.message_type,
                messages.timestamp,
                messages.upload_url,
                users.avatar_url
            FROM messages
            JOIN users ON messages.sender = users.username
            WHERE message_type = 'dm'
            AND (
                (LOWER(TRIM(sender)) = LOWER(TRIM($1)) AND LOWER(TRIM(target_username)) = LOWER(TRIM($2)))
                OR
                (LOWER(TRIM(sender)) = LOWER(TRIM($2)) AND LOWER(TRIM(target_username)) = LOWER(TRIM($1)))
            )
            ORDER BY timestamp ASC
            "#
        )
        .bind(current_user)
        .bind(target_user)
        .fetch_all(pool)
        .await?;
    println!("Checking DMs between: '{}' and '{}'", current_user, target_user);
    
        Ok(rows)
    }

}
