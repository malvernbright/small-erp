use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

use crate::{auth::Claims, AppState};

// --- Data Models ---

#[derive(Serialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub stock: i32,
    pub price: Decimal,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub stock: i32,
    pub price: Decimal,
}

#[derive(Deserialize)]
pub struct UpdateStockRequest {
    pub quantity_change: i32, // Positive to add stock, negative to remove
}

// --- Router Setup ---

pub fn inventory_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_products))
        .route("/", post(create_product))
        .route("/:id/stock", patch(update_stock))
}

// --- Route Handlers ---

async fn list_products(
    _claims: Claims, // Requires valid JWT, but all roles can view inventory
    State(state): State<AppState>,
) -> Result<Json<Vec<Product>>, (StatusCode, String)> {
    
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY id ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    Ok(Json(products))
}

async fn create_product(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    
    // RBAC: Only admins and warehouse managers can create products
    if claims.role != "admin" && claims.role != "warehouse_mgr" {
        return Err((StatusCode::FORBIDDEN, "Access Denied: Cannot create products".to_string()));
    }

    let product = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO products (sku, name, description, stock, price)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(&payload.sku)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.stock)
    .bind(payload.price)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create product: {}", e)))?;

    Ok(Json(product))
}

async fn update_stock(
    claims: Claims,
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateStockRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    
    // RBAC: Only admins and warehouse managers can manually adjust stock
    if claims.role != "admin" && claims.role != "warehouse_mgr" {
        return Err((StatusCode::FORBIDDEN, "Access Denied: Cannot update stock".to_string()));
    }

    let product = sqlx::query_as::<_, Product>(
        "UPDATE products SET stock = stock + $1 WHERE id = $2 RETURNING *"
    )
    .bind(payload.quantity_change)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update stock: {}", e)))?;

    Ok(Json(product))
}