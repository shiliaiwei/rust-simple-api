# Rust Simple API

Run: cargo run
Port: 3000

GET /  GitHub style root, pretty in browser, compact in terminal
GET /health
GET /api/categories
GET /api/items  seeded with 12 road sign lessons
POST /api/items  {"name":"Book","category":"General","code":"B1","price":12.5}
GET /api/items/:id
DELETE /api/items/:id
