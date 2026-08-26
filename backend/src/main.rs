use axum::{
    extract::State,
    http::{HeaderValue, Method},
    routing::get,
    Json, Router,
};
use dotenvy::dotenv;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tower_http::cors::CorsLayer;

// Define the module so Rust knows to compile auth.rs
mod auth;
mod inventory;
mod crm;
mod sales;

// Make the fields public so auth.rs can access the db pool
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");
    
    let state = AppState { db: pool };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .nest("/api/auth", auth::auth_routes())
        // Nest the protected inventory routes
        .nest("/api/inventory", inventory::inventory_routes())
        .nest("/api/crm", crm::crm_routes())
        .nest("/api/sales", sales::sales_routes())
        .layer(cors)
        .with_state(state);

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    println!("🚀 Backend server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "Database connected successfully",
        Err(_) => "Database connection failed",
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        message: db_status.to_string(),
    })
}