use futures_util::{StreamExt, SinkExt};
use axum::{
    extract::{Path, State},
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse
};
use serde_json::{json, Value};
use std::{
    collections::HashMap, path::PathBuf, sync::Arc
};
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

use crate::{db::get_pool, models::MessageModel};

pub struct ChatState {
    pub tx: broadcast::Sender<String>,
    pub users: HashMap<String, mpsc::UnboundedSender<Message>>, // uuid -> tx
    pub user_map: HashMap<String, String>,                      // uuid -> username
    pub upload_dir: PathBuf
    // pub rooms: HashMap<String, HashSet<String>>,                // room -> set of uuid
}

pub type SharedChatState = Arc<RwLock<ChatState>>;

impl ChatState {
    pub fn new() -> (Self, broadcast::Receiver<String>) {
        let (tx, rx) = broadcast::channel(100);
        (
            Self {
                tx,
                users: HashMap::new(),
                user_map: HashMap::new(),
                upload_dir: PathBuf::new()
                // rooms: HashMap::new(),
            },
            rx,
        )
    }
}

pub async fn handle_socket(
    Path(username): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<SharedChatState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, username, state))
}

async fn handle_connection(socket: WebSocket, username: String, state: SharedChatState) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let uuid = Uuid::new_v4().to_string();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    {
        let mut state = state.write().await;
        state.users.insert(uuid.clone(), tx.clone());
        state.user_map.insert(uuid.clone(), username.clone());
        println!("User '{}' connected with UUID {}", username, uuid);

        let _ = state.tx.send(json!({
            "type": "system",
            "message": format!("{} joined", username)
        }).to_string());
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let state_clone = Arc::clone(&state);
    let username_clone = username.clone();
    let uuid_clone = uuid.clone();
    let pool = get_pool().await;

    let mut recv_task = tokio::spawn(async move {

        while let Some(Ok(msg)) = ws_receiver.next().await
        {

            if let Message::Text(text) = msg 
            {
                if let Ok(data) = serde_json::from_str::<Value>(&text) 
                {
                    match data["type"].as_str() {
                        Some("dm") => {
                            let to_username = data["to"].as_str().unwrap_or("");
                            let message = data["message"].as_str().unwrap_or("");
                            let uploadurl = data["upload_url"].as_str().unwrap_or("").to_string();
                            let timestamp = chrono::Utc::now();

                            let state = state_clone.read().await;

                            let result = MessageModel::save_message(
                                pool, 
                                &username_clone, 
                                "dm",
                                message,
                                &timestamp, 
                                Some(to_username),
                                Some(uploadurl.clone())
                            ).await;

                            match result {
                                Ok(sav) => println!("{:?}", sav),
                                Err(e) => eprintln!("Error saving message: {:?}", e),
                            }

                            if let Some((recipient_uuid, _)) = state.user_map.iter().find(|(_, uname)| *uname == to_username) 
                            {
                                if let Some(tx) = state.users.get(recipient_uuid) 
                                {
                                    let _ = tx.send(Message::Text(
                                        json!({
                                            "type": "dm",
                                            "from": username_clone,
                                            "message": message,
                                            "upload_url": uploadurl
                                        }).to_string().into()
                                    ));
                                }
                            }
                        }
                        Some("chat") => {
                            let message = data["message"].as_str().unwrap_or("");
                            let uploadurl = data["upload_url"].as_str().unwrap_or("").to_string();
                            let to_username = Some(data["to"].as_str().unwrap_or(""));
                            let timestamp = chrono::Utc::now();

                            let result = MessageModel::save_message(
                                pool, 
                                &username_clone, 
                                "chat", 
                                message, 
                                &timestamp, 
                                to_username,
                                Some(uploadurl.clone())
                            ).await;

                            match result {
                                Ok(sav) => println!("{:?}", sav),
                                Err(e) => eprintln!("Error saving message: {:?}", e),
                            }

                            let _ = state_clone.read().await.tx.send(
                                json!({
                                    "type": "chat",
                                    "username": username_clone,
                                    "message": message,
                                    "upload_url": uploadurl
                                }).to_string()
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let mut broadcast_task = {
        let state = Arc::clone(&state);
        tokio::spawn(async move {

            let mut rx = state.read().await.tx.subscribe();

            while let Ok(msg) = rx.recv().await {
                if let Some(tx) = state.read().await.users.get(&uuid_clone) 
                {
                    let _ = tx.send(Message::Text(msg.into()));
                }
            }

        })
    };

    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
            broadcast_task.abort();
        },
        _ = &mut recv_task => {
            send_task.abort();
            broadcast_task.abort();
        },
        _ = &mut broadcast_task => {
            send_task.abort();
            recv_task.abort();
        },
    };

    let mut state = state.write().await;
    state.users.remove(&uuid);
    state.user_map.remove(&uuid);

    let _ = state.tx.send(json!({
        "type": "system",
        "message": format!("{} left", username)
    }).to_string());
    
    println!("User {} disconnected", uuid);
}
