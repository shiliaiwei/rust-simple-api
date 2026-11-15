use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::{Arc, RwLock}};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
struct Item { id: String, name: String, price: f64 }

#[derive(Deserialize)]
struct CreateItem { name: String, price: f64 }

type Store = Arc<RwLock<HashMap<String, Item>>>;

async fn home() -> Json<serde_json::Value> {
    Json(serde_json::json!({"success": true, "message": "Rust Simple API", "status": "online", "code": 200}))
}
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}
async fn list(State(s): State<Store>) -> Json<Vec<Item>> {
    Json(s.read().unwrap().values().cloned().collect())
}
async fn create(State(s): State<Store>, Json(p): Json<CreateItem>) -> (StatusCode, Json<Item>) {
    let item = Item { id: Uuid::new_v4().to_string(), name: p.name, price: p.price };
    s.write().unwrap().insert(item.id.clone(), item.clone());
    (StatusCode::CREATED, Json(item))
}
async fn get_one(State(s): State<Store>, Path(id): Path<String>) -> Result<Json<Item>, StatusCode> {
    s.read().unwrap().get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}
async fn remove(State(s): State<Store>, Path(id): Path<String>) -> StatusCode {
    if s.write().unwrap().remove(&id).is_some() { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
}

#[tokio::main]
async fn main() {
    let store: Store = Arc::new(RwLock::new(HashMap::new()));
    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .route("/api/items", get(list).post(create))
        .route("/api/items/:id", get(get_one).delete(remove))
        .layer(CorsLayer::permissive())
        .with_state(store);
    let port: u16 = std::env::var("PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://localhost:{port}");
    axum::serve(listener, app).await.unwrap();
}
