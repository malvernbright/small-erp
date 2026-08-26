use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{auth::Claims, AppState};

#[derive(Serialize, FromRow)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub fn users_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/:id/role", patch(update_role))
}

// Fetch all users (Admin only)
async fn list_users(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<UserProfile>>, (StatusCode, String)> {
    
    if claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Access Denied: Only admins can view users".to_string()));
    }

    let users = sqlx::query_as::<_, UserProfile>("SELECT id, username, role FROM users ORDER BY id ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    Ok(Json(users))
}

// Update a user's role (Admin only)
async fn update_role(
    claims: Claims,
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<Json<UserProfile>, (StatusCode, String)> {
    
    if claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Access Denied: Only admins can edit users".to_string()));
    }

    let user = sqlx::query_as::<_, UserProfile>(
        "UPDATE users SET role = $1 WHERE id = $2 RETURNING id, username, role"
    )
    .bind(&payload.role)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update role: {}", e)))?;

    Ok(Json(user))
}