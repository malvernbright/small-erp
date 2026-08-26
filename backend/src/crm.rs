use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{auth::Claims, AppState};

// --- Data Models ---

#[derive(Serialize, FromRow)]
pub struct Customer {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub email: Option<String>, // Option handles potential null values
    pub phone: Option<String>,
    pub address: Option<String>,
}

// --- Router Setup ---

pub fn crm_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_customers))
        .route("/", post(create_customer))
}

// --- Route Handlers ---

async fn list_customers(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<Customer>>, (StatusCode, String)> {
    
    // RBAC: Only admins and sales reps need to access customer data
    if claims.role != "admin" && claims.role != "sales_rep" {
        return Err((
            StatusCode::FORBIDDEN,
            "Access Denied: You do not have permission to view customers".to_string(),
        ));
    }

    let customers = sqlx::query_as::<_, Customer>("SELECT * FROM customers ORDER BY name ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    Ok(Json(customers))
}

async fn create_customer(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, String)> {
    
    // RBAC: Only admins and sales reps can create new customer profiles
    if claims.role != "admin" && claims.role != "sales_rep" {
        return Err((
            StatusCode::FORBIDDEN,
            "Access Denied: You do not have permission to create customers".to_string(),
        ));
    }

    let customer = sqlx::query_as::<_, Customer>(
        r#"
        INSERT INTO customers (name, email, phone, address)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.address)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        // Simple error handling to catch duplicate emails
        if e.to_string().contains("unique constraint") {
            (StatusCode::BAD_REQUEST, "Email already exists".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create customer: {}", e))
        }
    })?;

    Ok(Json(customer))
}