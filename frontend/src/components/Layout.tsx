import { Outlet, Link, useNavigate } from 'react-router-dom';
import { jwtDecode } from 'jwt-decode';

interface MyToken {
  sub: string;
  role: string;
  exp: number;
}

export default function Layout() {
  const navigate = useNavigate();
  const token = localStorage.getItem('token');
  
  let role = '';
  let username = '';
  
  if (token) {
    try {
      const decoded = jwtDecode<MyToken>(token);
      role = decoded.role;
      username = decoded.sub;
    } catch (e: unknown) {
      console.error("Invalid token: ", (e as Error).toString());
    }
  }

  // Odoo-style Access Booleans
  const isAdmin = role === 'admin';
  const canViewInventory = isAdmin || role.includes('inventory');
  const canViewCrm = isAdmin || role.includes('crm');
  const canViewSales = isAdmin || role.includes('sales');

  const handleLogout = () => {
    localStorage.removeItem('token');
    navigate('/login');
  };

  return (
    <div style={{ display: 'flex', minHeight: '100vh', fontFamily: 'sans-serif' }}>
      {/* Sidebar Navigation */}
      <div style={{ width: '220px', background: '#2c3e50', color: 'white', display: 'flex', flexDirection: 'column' }}>
        <div style={{ padding: '20px', background: '#1a252f' }}>
          <h2 style={{ margin: 0 }}>Mini ERP</h2>
          <p style={{ margin: '5px 0 0 0', fontSize: '12px', color: '#18bc9c' }}>Logged in as: {username}</p>
        </div>
        
        <nav style={{ display: 'flex', flexDirection: 'column', padding: '20px', gap: '15px', flex: 1 }}>
          <Link to="/" style={{ color: 'white', textDecoration: 'none' }}>📊 Dashboard</Link>
          
          {/* Conditionally Render Apps Based on Ticked Access */}
          {canViewInventory && <Link to="/inventory" style={{ color: 'white', textDecoration: 'none' }}>📦 Inventory</Link>}
          {canViewCrm && <Link to="/crm" style={{ color: 'white', textDecoration: 'none' }}>👥 CRM (Customers)</Link>}
          {canViewSales && <Link to="/sales" style={{ color: 'white', textDecoration: 'none' }}>🛒 Sales</Link>}
          
          {/* Only Admins can manage users */}
          {isAdmin && (
            <div style={{ marginTop: 'auto', borderTop: '1px solid #444', paddingTop: '15px' }}>
              <Link to="/users" style={{ color: '#f39c12', textDecoration: 'none' }}>⚙️ Manage Users</Link>
            </div>
          )}
        </nav>
        
        <button 
          onClick={handleLogout} 
          style={{ padding: '15px', background: '#e74c3c', color: 'white', border: 'none', cursor: 'pointer', fontWeight: 'bold' }}
        >
          Logout
        </button>
      </div>

      {/* Main Content Area */}
      <div style={{ flex: 1, padding: '30px', background: '#ecf0f1' }}>
        <Outlet />
      </div>
    </div>
  );
}