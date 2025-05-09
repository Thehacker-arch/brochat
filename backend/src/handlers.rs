// handlers.rs
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::{Multipart, Path};
use axum::Extension;
use axum::{extract::State, http::StatusCode, Json};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::models::AuthenticatedUser;
use crate::ws::ChatState;
use crate::{auth::create_jwt, db::get_pool, models::{MessageModel, User}, utils::{hash_password, verify_password}, ws::SharedChatState};
use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};


#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}


pub async fn register(Json(payload): Json<AuthPayload>) -> Result<Json<User>, (StatusCode, Json<Value>)> {

    let pool = get_pool().await;
    let hashed = hash_password(&payload.password)
        .map_err(|_e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Password hashing failed"})),
            )
        })?;

    match User::create(&pool, &payload.username, &hashed).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            eprintln!("Registration failed: {:?}", e);
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return Err((
                        StatusCode::CONFLICT,
                        Json(json!({
                            "status": "error",
                            "message": "Username already exists !!"
                        })),
                    ));
                }
            }
            Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": "Registration Failed"
                })),
            ))
        }
    }
}


pub async fn get_meapi(
    Extension(user): Extension<AuthenticatedUser>
) -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "id": user.id,
            "username": user.username,
            "avatar_url": user.avatar_url
        }))
    )
}


pub async fn login(
    Json(payload): Json<AuthPayload>
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = get_pool().await;

    let user = match User::find_by_username(&pool, &payload.username).await {
        Ok(user) => user,
        Err(e) => {
            eprintln!("ERROR {}", e);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "error",
                    "message": "Invalid credentials"
                })),
            ))
        }
    };

    // Verify password
    match verify_password(&payload.password, &user.password_hash) {
        Ok(true) => {
            let token = create_jwt(&user).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": "error", "message": "Token generation failed"})),
                )
            })?;
            
            Ok(Json(json!({
                "status": "success",
                "token": token,
                "user_id": user.id,
                "username": user.username
            })))
        }
        Ok(false) => {
            Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "status": "error",
                    "message": "Invalid credentials"
                })),
            ))
        }
        Err(_) => {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "message": "Password verification failed"})),
            ))
        }
    }
}


pub async fn list_users() -> impl IntoResponse {
   let pool = get_pool().await;

   match User::find_all(&pool).await {
        Ok(users) => {
            (StatusCode::OK, Json(users)).into_response()
        }

        Err(e) => {
            eprintln!("Error in fetching users: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database query failed"}))).into_response()
        }
    }
}

pub async fn get_dms(State(state): State<SharedChatState>) -> Json<serde_json::Value> {
    let state = state.read().await;
    // Deduplicate usernames using a HashSet
    let usernames: HashSet<String> = state.user_map.values().cloned().collect();
    Json(json!({ "dms": usernames }))
}

pub async fn get_public_messages() -> impl IntoResponse {
    let pool = get_pool().await;

    match MessageModel::get_public_messages(&pool).await {
        Ok(messages) => (StatusCode::OK, Json(messages)).into_response(),
        Err(e) => {
            eprintln!("DB error in public message: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database query failed"}))).into_response()
        }
    }
}


pub async fn auth_middleware(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let pool = get_pool().await;
    let headers = req.headers();
    println!("Headers: {:?}", headers);

    let user = AuthenticatedUser::from_auth_header(headers.clone(), pool)
        .await.map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}


pub async fn get_dm_messages(
    Path(target_user): Path<String>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    let pool = get_pool().await;
    let current_user = &auth_user.username;


    if current_user != &target_user {
        // We'll check if there's a DM between current_user and target_user
        // This logic assumes both sides can see the conversation
        match MessageModel::get_dm_messages(&pool, current_user, &target_user).await {
            Ok(messages) =>
            { 
                        if messages.is_empty() {
                            println!("No messages found.");
                        } else {
                            println!("Messages found: {:?}", messages);
                        }
                        println!(" MESSAGES b/w {} {} {:?}", current_user, &target_user,&messages.to_owned());
                        (StatusCode::OK, Json(messages)).into_response()
                
            }
            Err(e) => {
                eprintln!("DB error in dm messages: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Database query failed" }))
                ).into_response()
            }
        }
    } 
    else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Cannot load DMs with yourself" }))
        ).into_response()
    }
}



pub async fn handle_uploads(
    State(state): State<Arc<RwLock<ChatState>>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let state = state.read().await; 
    let mut filename = None;
    let mut file_data = None;

    // Go through all form fields
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "file" => {
                let original_name = field.file_name().unwrap_or("file").to_string();
                let pat = PathBuf::from(&original_name);
                  let ext= pat.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                let unique_name = format!("{}_{}.{}", Uuid::new_v4(), chrono::Utc::now().timestamp(), ext);

                let data = field.bytes().await.unwrap();
                filename = Some(unique_name);
                file_data = Some(data);
            }
            "sender" => {
                let _sender = field.text().await.unwrap_or_default();
            }
            "chat" => {
                let _chat = field.text().await.unwrap_or_default();
            }
            _ => {}
        }
    }

    if let (Some(name), Some(data)) = (filename, file_data) {
        let upload_dir = PathBuf::from(&state.upload_dir).join("uploads");

        if let Err(e) = tokio::fs::create_dir_all(&upload_dir).await {
            eprintln!("Failed to create avatar dir: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Could not create avatar folder" })),
            );
        }

        let full_path = upload_dir.join(&name);
        let upload_url = format!("/uploads/{}", name);
        println!("{}", upload_url.clone());
        match tokio::fs::write(&full_path, &data).await {
            Ok(_) => (
                StatusCode::OK,
                Json(serde_json::json!({ 
                    "status": "success", 
                    "filename": name,
                    "upload_url": upload_url
                }))
            ),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": err.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing file data" })),
        )
    }
}


pub async fn handle_avatar(
    State(state): State<Arc<RwLock<ChatState>>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let pool = get_pool().await;
    let state = state.read().await;
    let mut filename = None;
    let mut file_data = None;
    let mut id: Option<Uuid> = None;

    while let Some(field_result) = multipart.next_field().await.transpose() {
        let field = match field_result {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Multipart read error: {:?}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Failed to read multipart form data" })),
                );
            }
        };

        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "avatar" => {
                let original_name = field.file_name().unwrap_or("file").to_string();
                let path_buf = PathBuf::from(&original_name);
                let ext = path_buf.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                let unique_name = format!(
                    "{}_{}.{}",
                    Uuid::new_v4(),
                    chrono::Utc::now().timestamp(),
                    ext
                );

                let data = match field.bytes().await {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Failed to read file bytes: {:?}", e);
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "error": "Failed to read uploaded file" })),
                        );
                    }
                };

                filename = Some(unique_name);
                file_data = Some(data);
            }

            "user_id" => {
                let val = match field.text().await {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Failed to read user_id field: {:?}", e);
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "error": "Invalid user ID" })),
                        );
                    }
                };

                eprintln!("Got user_id from multipart: {:?}", val);

                match Uuid::parse_str(val.trim()) {
                    Ok(uuid) => id = Some(uuid),
                    Err(_) => {
                        eprintln!("Invalid UUID: {}", val);
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(json!({ "error": "Invalid user ID" })),
                        );
                    }
                }
            }

            _ => {}
        }
    }

    eprintln!(
        "filename: {:?}, file_data: {:?}, id: {:?}",
        filename,
        file_data.as_ref().map(|f| f.len()),
        id
    );

    if let (Some(name), Some(data), Some(uid)) = (filename, file_data, id) {
        let avatar_dir = PathBuf::from(&state.upload_dir).join("avatars");

        if let Err(e) = tokio::fs::create_dir_all(&avatar_dir).await {
            eprintln!("Failed to create avatar dir: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Could not create avatar folder" })),
            );
        }

        let full_path = avatar_dir.join(&name);

        match tokio::fs::write(&full_path, &data).await {
            Ok(_) => {
                let avatar_url = format!("/avatars/{}", name);
                println!("{}", avatar_url.clone());
                match User::insert_pfp(pool, Some(uid), avatar_url.clone()).await {
                    Ok(_) => (
                        StatusCode::OK,
                        Json(json!({ "status": "success", "filename": name, "avatarUrl": avatar_url})),
                    ),
                    Err(e) => {
                        eprintln!("DB update error: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": "Database update failed" })),
                        )
                    }
                }
            }

            Err(e) => {
                eprintln!("Failed to write file: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to write avatar to disk" })),
                )
            }
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing avatar or user ID" })),
        )
    }
}
