use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

use crate::{auth::Claims, AppState};

// --- Data Models ---

#[derive(Deserialize)]
pub struct OrderItemRequest { pub product_id: i32, pub quantity: i32 }

#[derive(Deserialize)]
pub struct CreateOrderRequest { pub customer_id: i32, pub items: Vec<OrderItemRequest> }

#[derive(Serialize)]
pub struct OrderResponse { pub order_id: i32, pub total_amount: Decimal, pub status: String, pub message: String }

#[derive(Serialize)]
pub struct OrderDetail {
    pub id: i32,
    pub customer_name: String,
    pub total_amount: Decimal,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub items: Vec<OrderLineDetail>,
}

#[derive(Serialize, FromRow)]
pub struct OrderLineDetail {
    pub product_name: String,
    pub sku: String,
    pub quantity: i32,
    pub unit_price: Decimal,
}

#[derive(Serialize, FromRow)]
pub struct OrderSummary {
    pub id: i32,
    pub customer_name: String,
    pub total_amount: Decimal,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

// --- Router Setup ---

pub fn sales_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_orders))        // NEW: Fetch all orders
        .route("/", post(create_sales_order))
        .route("/:id", get(get_order_detail)) // NEW: Fetch single order for PDF
}

// --- Route Handlers ---

async fn list_orders(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<OrderSummary>>, (StatusCode, String)> {
    if claims.role != "admin" && !claims.role.contains("sales") {
        return Err((StatusCode::FORBIDDEN, "Access Denied".to_string()));
    }

    let orders = sqlx::query_as::<_, OrderSummary>(
        r#"
        SELECT o.id, c.name as customer_name, o.total_amount, o.status, o.created_at
        FROM sales_orders o
        JOIN customers c ON o.customer_id = c.id
        ORDER BY o.id DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    Ok(Json(orders))
}

async fn get_order_detail(
    claims: Claims,
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<OrderDetail>, (StatusCode, String)> {
    if claims.role != "admin" && !claims.role.contains("sales") {
        return Err((StatusCode::FORBIDDEN, "Access Denied".to_string()));
    }

    // 1. Get the parent order
    let order_summary = sqlx::query_as::<_, OrderSummary>(
        "SELECT o.id, c.name as customer_name, o.total_amount, o.status, o.created_at FROM sales_orders o JOIN customers c ON o.customer_id = c.id WHERE o.id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
    .ok_or((StatusCode::NOT_FOUND, "Order not found".to_string()))?;

    // 2. Get the line items
    let items = sqlx::query_as::<_, OrderLineDetail>(
        r#"
        SELECT p.name as product_name, p.sku, i.quantity, i.unit_price
        FROM sales_order_items i
        JOIN products p ON i.product_id = p.id
        WHERE i.order_id = $1
        "#
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    Ok(Json(OrderDetail {
        id: order_summary.id,
        customer_name: order_summary.customer_name,
        total_amount: order_summary.total_amount,
        status: order_summary.status,
        created_at: order_summary.created_at,
        items,
    }))
}

async fn create_sales_order(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<OrderResponse>, (StatusCode, String)> {
    
    // 1. RBAC: Only admins and sales reps can create orders
    if claims.role != "admin" && !claims.role.contains("sales") {
        return Err((StatusCode::FORBIDDEN, "Access Denied: Only sales reps can create orders".to_string()));
    }

    // 2. We need the numeric user_id for the database, but our JWT only holds the username.
    let user_row = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(&claims.sub)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;
    let user_id: i32 = user_row.get("id");

    // ==========================================
    // START DATABASE TRANSACTION
    // ==========================================
    let mut tx = state.db.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. Create the parent Sales Order (we will update the total_amount at the end)
    let order_row = sqlx::query(
        "INSERT INTO sales_orders (customer_id, user_id, total_amount, status) VALUES ($1, $2, 0, 'completed') RETURNING id"
    )
    .bind(payload.customer_id)
    .bind(user_id)
    .fetch_one(&mut *tx) // Pass the transaction instead of the pool
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create order: {}", e)))?;
    
    let order_id: i32 = order_row.get("id");
    let mut total_amount = Decimal::new(0, 0);

    // 4. Process each item in the order
    for item in payload.items {
        
        // Fetch product and lock the row (FOR UPDATE) so no one else can change the stock while we process this
        let product_row = sqlx::query("SELECT stock, price FROM products WHERE id = $1 FOR UPDATE")
            .bind(item.product_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
            .ok_or((StatusCode::BAD_REQUEST, format!("Product ID {} not found", item.product_id)))?;

        let current_stock: i32 = product_row.get("stock");
        let price: Decimal = product_row.get("price");

        // 5. Check if we have enough stock
        if current_stock < item.quantity {
            // Returning an error automatically drops `tx`, rolling back everything we've done so far!
            return Err((StatusCode::BAD_REQUEST, format!("Insufficient stock for Product ID {}. Only {} left.", item.product_id, current_stock)));
        }

        // 6. Deduct the stock
        sqlx::query("UPDATE products SET stock = stock - $1 WHERE id = $2")
            .bind(item.quantity)
            .bind(item.product_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update stock: {}", e)))?;

        // 7. Insert the order line item
        sqlx::query(
            "INSERT INTO sales_order_items (order_id, product_id, quantity, unit_price) VALUES ($1, $2, $3, $4)"
        )
        .bind(order_id)
        .bind(item.product_id)
        .bind(item.quantity)
        .bind(price)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to insert order item: {}", e)))?;

        // 8. Accumulate the total cost
        total_amount += price * rust_decimal::Decimal::from(item.quantity);
    }

    // 9. Finalize the order by updating it with the calculated grand total
    sqlx::query("UPDATE sales_orders SET total_amount = $1 WHERE id = $2")
        .bind(total_amount)
        .bind(order_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update order total: {}", e)))?;

    // ==========================================
    // COMMIT TRANSACTION
    // ==========================================
    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Transaction commit failed: {}", e)))?;

    Ok(Json(OrderResponse {
        order_id,
        total_amount,
        status: "completed".to_string(),
        message: "Order placed and stock updated successfully.".to_string(),
    }))
}