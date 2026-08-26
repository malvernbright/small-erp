-- Users table (Authentication & RBAC)
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role VARCHAR(50) NOT NULL, -- 'admin', 'sales_rep', 'warehouse_mgr'
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Customers table (CRM)
CREATE TABLE customers (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,
    phone VARCHAR(50),
    address TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Products table (Inventory)
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    sku VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    stock INT NOT NULL DEFAULT 0,
    price NUMERIC(10, 2) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Sales Orders table (Sales core)
CREATE TABLE sales_orders (
    id SERIAL PRIMARY KEY,
    customer_id INT NOT NULL REFERENCES customers(id) ON DELETE RESTRICT,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    total_amount NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    status VARCHAR(50) DEFAULT 'completed',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Sales Order Items (Links orders to products for the ERP)
CREATE TABLE sales_order_items (
    id SERIAL PRIMARY KEY,
    order_id INT NOT NULL REFERENCES sales_orders(id) ON DELETE CASCADE,
    product_id INT NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    quantity INT NOT NULL,
    unit_price NUMERIC(10, 2) NOT NULL
);