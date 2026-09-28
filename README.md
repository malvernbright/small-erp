# Mini-ERP System (Rust + React)

A full-stack, modular Enterprise Resource Planning (ERP) system featuring Role-Based Access Control (RBAC), CRM, Inventory Management, and a transactional Sales Order system.

The backend is built with **Rust (Axum + SQLx)** and the frontend is powered by **React (Vite + TypeScript)**.

## Prerequisites

Ensure you have the following installed on your system:

* **Rust & Cargo** (v1.70+)
* **Node.js** (v18+) & **npm**
* **PostgreSQL** (running locally or remotely)

### Install SQLx CLI

The backend relies on SQLx for database migrations. You must install the `sqlx-cli` tool globally via Cargo before setting up the database.

Run the following command in your terminal:

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres

```

---

## 1. Backend Setup & Database Creation

1. **Navigate to the backend directory:**
```bash
cd backend

```


2. **Configure Environment Variables:**
Create a `.env` file in the root of the `backend` directory. Adjust the `DATABASE_URL` to match your PostgreSQL credentials.
```env
DATABASE_URL=postgres://postgres:password@localhost/small_erp
JWT_SECRET=your_super_secret_jwt_key_here

```


3. **Initialize the Database:**
Use the `sqlx` CLI tool to create the database and run the initial migration scripts to build the tables.
```bash
sqlx database create
sqlx migrate run

```


4. **Start the Backend Server:**
```bash
cargo run

```


*The server will start on `[http://127.0.0.1:3000](http://127.0.0.1:3000)`.*

---

## 2. Frontend Setup

1. **Navigate to the frontend directory:**
Open a new terminal window/tab and navigate to the frontend folder.
```bash
cd frontend

```


2. **Install Dependencies:**
```bash
npm install

```


3. **Start the Development Server:**
```bash
npm run dev

```


*The frontend will be available at `http://localhost:5173`.*

---

## 3. Initial Bootstrapping (Creating the First Admin)

Because the system uses strict Role-Based Access Control, you need an initial Superuser (Admin) account to access the UI and create other employees.

With your backend server running, execute the following `cURL` command in a terminal to register the first admin account:

```bash
curl -X POST http://127.0.0.1:3000/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"username": "admin", "password": "password123", "role": "admin"}'

```

Once executed, open `http://localhost:5173` in your browser, log in with `admin` and `password123`, and you will have full access to the system.

---

## Features & Modules

* **Authentication & RBAC:** JWT-based stateless authentication. Odoo-style modular access allows ticking specific applications (CRM, Inventory, Sales) for individual users, dynamically rendering the sidebar.
* **CRM (Customer Relationship Management):** Track customer names, emails, phones, and addresses.
* **Inventory Management:** Create products, define SKUs, set prices, and adjust stock levels safely.
* **Sales Orders & Invoicing:**
* Add multiple items to a cart mapped to a specific customer.
* **Database Transactions:** Orders are processed inside a strict PostgreSQL transaction. If an order exceeds available stock, the entire transaction rolls back safely to prevent data corruption.
* **PDF Invoices:** Dedicated invoice view dynamically strips out UI elements (sidebars, buttons) utilizing CSS `@media print` to trigger a clean, native browser PDF print dialog.