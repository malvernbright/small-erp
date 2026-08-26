use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;


use crate::AppState;

// --- Data Models ---

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub role: String, // e.g., "admin", "sales_rep", "warehouse_mgr"
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub role: String,
    pub exp: usize,  // expiration timestamp
}

// Database representation of a User
#[derive(sqlx::FromRow)]
struct UserRow {
    id: i32,
    username: String,
    password_hash: String,
    role: String,
}

// --- Router Setup ---

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

// --- Route Handlers ---

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 1. Hash the password using Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Error hashing password: {}", e)))?
        .to_string();

    // 2. Insert into the database
    let query = "INSERT INTO users (username, password_hash, role) VALUES ($1, $2, $3) RETURNING id";
    let result = sqlx::query(query)
        .bind(&payload.username)
        .bind(&password_hash)
        .bind(&payload.role)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Ok(Json(AuthResponse {
            token: "User registered successfully! Please login.".to_string(),
        })),
        Err(_) => Err((
            StatusCode::BAD_REQUEST,
            "Username already exists or database error.".to_string(),
        )),
    }
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // 1. Fetch user from the database
    let query = "SELECT id, username, password_hash, role FROM users WHERE username = $1";
    let user = sqlx::query_as::<_, UserRow>(query)
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))?;

    let user = match user {
        Some(u) => u,
        None => return Err((StatusCode::UNAUTHORIZED, "Invalid username or password".to_string())),
    };

    // 2. Verify the submitted password against the stored hash
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid password hash format".to_string()))?;
        
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()))?;

    // 3. Generate a JWT token if verification passes
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.username,
        role: user.role,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error".to_string()))?;

    Ok(Json(AuthResponse { token }))
}

// --- Axum Extractor for JWT Authentication ---

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Grab the "Authorization" header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

        // 2. Ensure it starts with "Bearer "
        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid Authorization header format".to_string()));
        }

        let token = &auth_header["Bearer ".len()..];

        // 3. Decode and validate the JWT
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(), // Default validation checks token expiration automatically
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string()))?;

        // 4. Return the parsed claims to the route handler
        Ok(token_data.claims)
    }
}