import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom';
import type { ReactNode } from 'react';
import Login from './pages/Login';
import Crm from './pages/Crm';
import Users from './pages/Users';
import Inventory from './pages/Inventory';
import Layout from './components/Layout';

function ProtectedRoute({ children }: { children: ReactNode }) {
  const token = localStorage.getItem('token');
  if (!token) {
    return <Navigate to="/login" replace />;
  }
  return <>{children}</>;
}

// A simple home screen component
function Home() {
  return (
    <div>
      <h2>Dashboard Overview</h2>
      <p>Welcome to your ERP system. Select a module from the sidebar.</p>
    </div>
  );
}

const router = createBrowserRouter([
  {
    path: "/login",
    element: <Login />,
  },
  {
    path: "/",
    element: (
      <ProtectedRoute>
        <Layout />
      </ProtectedRoute>
    ),
    children: [
      { index: true, element: <Home /> }, 
      { path: "inventory", element: <Inventory /> },
      { path: "crm", element: <Crm /> },       // <-- Add CRM route
      { path: "users", element: <Users /> },   // <-- Add Users route
      { path: "sales", element: <div>Sales coming soon...</div> },
    ]
  },
]);

function App() {
  return <RouterProvider router={router} />;
}

export default App;