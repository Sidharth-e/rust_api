use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize, Deserialize)]
struct Item {
    id: u32,
    name: String,
}

#[derive(Deserialize)]
struct CreateItem {
    name: String,
}

type SharedState = Arc<Mutex<Vec<Item>>>;

#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(Mutex::new(vec![Item {
        id: 1,
        name: "First Item".to_string(),
    }]));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/items", get(list_items).post(create_item))
        .route("/items/{id}", get(get_item))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_items(State(state): State<SharedState>) -> Json<Vec<Item>> {
    let items = state.lock().unwrap();
    Json(items.clone())
}

async fn create_item(
    State(state): State<SharedState>,
    Json(payload): Json<CreateItem>,
) -> (StatusCode, Json<Item>) {
    let mut items = state.lock().unwrap();
    let new_item = Item {
        id: (items.len() as u32) + 1,
        name: payload.name,
    };
    items.push(new_item.clone());
    (StatusCode::CREATED, Json(new_item))
}

async fn get_item(
    State(state): State<SharedState>,
    Path(id): Path<u32>,
) -> Result<Json<Item>, StatusCode> {
    let items = state.lock().unwrap();
    items
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}