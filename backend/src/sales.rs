use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{auth::Claims, AppState};

// --- Data Models ---

#[derive(Deserialize)]
pub struct OrderItemRequest {
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: i32,
    pub items: Vec<OrderItemRequest>,
}

#[derive(Serialize)]
pub struct OrderResponse {
    pub order_id: i32,
    pub total_amount: Decimal,
    pub status: String,
    pub message: String,
}

// --- Router Setup ---

pub fn sales_routes() -> Router<AppState> {
    Router::new().route("/", post(create_sales_order))
}

// --- Route Handlers ---

async fn create_sales_order(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<OrderResponse>, (StatusCode, String)> {
    
    // 1. RBAC: Only admins and sales reps can create orders
    if claims.role != "admin" && claims.role != "sales_rep" {
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