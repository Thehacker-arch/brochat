mod auth;
mod db;
mod handlers;
mod models;
mod utils;
mod ws;

use axum::http::header;
use axum::{middleware, Extension};
use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, post},
};
use handlers::auth_middleware;
use std::{env, net::SocketAddr};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use ws::ChatState;


#[tokio::main]
async fn main() {
    if let Ok(database_url) = env::var("DATABASE_URL") {
        println!("Connected to: {}", database_url);
    } else {
        println!("DATABASE_URL is not set");
    }

    if let Err(e) = db::init().await {
        eprintln!("Database initialization failed: {}", e);
        return;
    }

    let (chat_state, _rx) = ChatState::new();
    let shared_state = Arc::new(RwLock::new(chat_state));


    let public_routes = Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/ws/{username}", get(ws::handle_socket))
        .route("/users", get(handlers::list_users))
        .route("/public", get(handlers::get_public_messages))
        .with_state(shared_state.clone());


    let protected_routes = Router::new()
        .route("/dms", get(handlers::get_dms))
        .route("/dm/{target_user}", get(handlers::get_dm_messages))
        .route("/me", get(handlers::get_meapi))
        .route("/upload", post(handlers::handle_uploads))
        .layer(middleware::from_fn(auth_middleware))
        .route("/avatar-upload", post(handlers::handle_avatar))
        .layer(middleware::from_fn(auth_middleware));


    let app = Router::new()
        .merge(public_routes)
        .nest("/api", protected_routes)
        .nest_service("/avatars", ServeDir::new("avatars"))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .with_state(shared_state.clone())
        .layer(Extension(shared_state))
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                    "http://192.168.1.45:5173".parse::<HeaderValue>().unwrap(),
                    "http://tauri.localhost".parse::<HeaderValue>().unwrap(),
                    "https://brochat.duckdns.org".parse::<HeaderValue>().unwrap()
                ])
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("[+] Listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
