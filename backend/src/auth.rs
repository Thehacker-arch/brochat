use std::env;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub username: String,
    avatar_url: Option<String>
}

pub fn create_jwt(user: &crate::models::User) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id,
        exp: expiration as usize,
        avatar_url: Some(user.avatar_url.clone()),
        username: user.username.clone()
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    
}


pub fn decode_jwt(token: &str) -> Result<Claims, String> {
    let secret = env::var("JWT_SECRET")
        .map_err(|_| "JWT_SECRET not set in environment".to_string())?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token expired".to_string(),
        jsonwebtoken::errors::ErrorKind::InvalidToken => "Invalid token".to_string(),
        _ => format!("Token validation failed: {}", e),
    })
}