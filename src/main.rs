use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::{Arc, RwLock}};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    category: String,
    code: String,
    price: f64,
}

#[derive(Deserialize)]
struct CreateItem {
    name: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    price: f64,
}

type Store = Arc<RwLock<HashMap<String, Item>>>;

fn seed(store: &Store) {
    let data = [
        ("Hospital Ahead", "Informative Signs", "PW03-G4-01"),
        ("Petrol Station", "Informative Signs", "PW03-G4-02"),
        ("Parking Place", "Informative Signs", "PW03-G4-03"),
        ("Restaurant", "Informative Signs", "PW03-G4-04"),
        ("Hotel", "Informative Signs", "PW03-G4-05"),
        ("Sharp Bend Ahead", "Warning Signs", "PW03-W1-01"),
        ("Pedestrian Crossing Ahead", "Warning Signs", "PW03-W1-02"),
        ("Slippery Road", "Warning Signs", "PW03-W1-03"),
        ("Stop", "Priority Signs", "PW03-P1-01"),
        ("Give Way", "Priority Signs", "PW03-P1-02"),
        ("No Entry", "Prohibitory Signs", "PW03-R1-01"),
        ("No Parking", "Prohibitory Signs", "PW03-R1-02"),
    ];
    let mut s = store.write().unwrap();
    for (name, category, code) in data {
        let id = Uuid::new_v4().to_string();
        s.insert(id.clone(), Item {
            id,
            name: name.to_string(),
            category: category.to_string(),
            code: code.to_string(),
            price: 0.0,
        });
    }
}

fn base_url(headers: &HeaderMap) -> String {
    let host = headers.get("host").and_then(|v| v.to_str().ok()).unwrap_or("localhost:3000");
    let proto = headers.get("x-forwarded-proto").and_then(|v| v.to_str().ok())
        .unwrap_or(if host.starts_with("localhost") || host.starts_with("127.") { "http" } else { "https" });
    format!("{proto}://{host}")
}

async fn home(State(s): State<Store>, headers: HeaderMap) -> Response {
    let base = base_url(&headers);
    let count = s.read().unwrap().len();
    let body = serde_json::json!({
        "success": true,
        "message": "Rust Simple API",
        "status": "online",
        "code": 200,
        "version": "0.2.0",
        "items_url": format!("{base}/api/items"),
        "item_url": format!("{base}/api/items/{{id}}"),
        "categories_url": format!("{base}/api/categories"),
        "health_url": format!("{base}/health"),
        "repository_url": "https://github.com/shiliaiwei/rust-simple-api",
        "database": {"type": "seeded", "items": count},
        "credits": "Kesararam With Digital"
    });
    let accept = headers.get("accept").and_then(|v| v.to_str().ok()).unwrap_or("");
    if accept.contains("text/html") {
        let pretty = serde_json::to_string_pretty(&body).unwrap();
        ([(axum::http::header::CONTENT_TYPE, "application/json")], pretty).into_response()
    } else {
        Json(body).into_response()
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn categories(State(s): State<Store>) -> Json<Vec<serde_json::Value>> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for item in s.read().unwrap().values() {
        *counts.entry(item.category.clone()).or_default() += 1;
    }
    let mut out: Vec<serde_json::Value> = counts.iter()
        .map(|(name, count)| serde_json::json!({"name": name, "items": count}))
        .collect();
    out.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Json(out)
}

async fn list(State(s): State<Store>) -> Json<Vec<Item>> {
    let mut items: Vec<Item> = s.read().unwrap().values().cloned().collect();
    items.sort_by(|a, b| a.code.cmp(&b.code));
    Json(items)
}

async fn create(State(s): State<Store>, Json(p): Json<CreateItem>) -> (StatusCode, Json<Item>) {
    let item = Item {
        id: Uuid::new_v4().to_string(),
        name: p.name,
        category: if p.category.is_empty() { "General".to_string() } else { p.category },
        code: p.code,
        price: p.price,
    };
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
    seed(&store);
    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .route("/api/categories", get(categories))
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
