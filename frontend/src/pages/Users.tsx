import { useState, useEffect } from 'react';
import api from '../api';

interface UserProfile {
  id: number;
  username: string;
  role: string;
}

export default function Users() {
  const [userList, setUserList] = useState<UserProfile[]>([]);
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  
  // Form State
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  
  // Odoo-Style Checkboxes
  const [isAdmin, setIsAdmin] = useState(false);
  const [accessCrm, setAccessCrm] = useState(false);
  const [accessInventory, setAccessInventory] = useState(false);
  const [accessSales, setAccessSales] = useState(false);

  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    let isMounted = true;
    const fetchUsers = async () => {
      try {
        const response = await api.get('/users');
        if (isMounted) setUserList(response.data);
      } catch (err: unknown) {
        if (isMounted) setError('Failed to fetch users. ' + (err as Error).toString());
      }
    };
    fetchUsers();
    return () => { isMounted = false; };
  }, [refreshTrigger]);

  // Helper to bundle checkboxes into our database string
  const getRoleString = () => {
    if (isAdmin) return 'admin';
    const apps = [];
    if (accessCrm) apps.push('crm');
    if (accessInventory) apps.push('inventory');
    if (accessSales) apps.push('sales');
    return apps.length > 0 ? apps.join(',') : 'none';
  };

  const handleCreateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    setMessage('');
    setError('');

    try {
      await api.post('/auth/register', { 
        username, 
        password, 
        role: getRoleString() 
      });
      setMessage(`User ${username} successfully created!`);
      setUsername('');
      setPassword('');
      setIsAdmin(false);
      setAccessCrm(false);
      setAccessInventory(false);
      setAccessSales(false);
      setRefreshTrigger(prev => prev + 1);
    } catch (err: unknown) {
      setError('Failed to create user. Please try again. ' + (err as Error).toString());
    }
  };

  return (
    <div>
      <h2>Access Rights Management</h2>
      <p style={{ color: '#666', fontSize: '14px', marginBottom: '20px' }}>
        Tick the applications each employee is allowed to view and manage.
      </p>

      {message && <p style={{ color: 'green', padding: '10px', background: '#e6ffe6' }}>{message}</p>}
      {error && <p style={{ color: 'red', padding: '10px', background: '#ffe6e6' }}>{error}</p>}

      <div style={{ display: 'flex', gap: '30px', alignItems: 'flex-start', flexWrap: 'wrap' }}>
        
        {/* ODOO-STYLE APP TICKER FORM */}
        <form onSubmit={handleCreateUser} style={{ flex: '1', minWidth: '300px', background: '#fff', padding: '20px', borderRadius: '5px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3>Create New User</h3>
          
          <div style={{ marginBottom: '15px' }}>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: 'bold' }}>Username</label>
            <input type="text" value={username} onChange={e => setUsername(e.target.value)} required style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }} />
          </div>
          
          <div style={{ marginBottom: '20px' }}>
            <label style={{ display: 'block', fontSize: '12px', fontWeight: 'bold' }}>Temporary Password</label>
            <input type="password" value={password} onChange={e => setPassword(e.target.value)} required style={{ width: '100%', padding: '8px', boxSizing: 'border-box' }} />
          </div>

          <h4 style={{ borderBottom: '1px solid #eee', paddingBottom: '5px' }}>Application Access</h4>
          
          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', marginBottom: '20px' }}>
            <label style={{ display: 'flex', alignItems: 'center', gap: '10px', cursor: 'pointer' }}>
              <input type="checkbox" checked={accessCrm} onChange={e => setAccessCrm(e.target.checked)} disabled={isAdmin} />
              CRM (Customers)
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: '10px', cursor: 'pointer' }}>
              <input type="checkbox" checked={accessSales} onChange={e => setAccessSales(e.target.checked)} disabled={isAdmin} />
              Sales Orders
            </label>
            <label style={{ display: 'flex', alignItems: 'center', gap: '10px', cursor: 'pointer' }}>
              <input type="checkbox" checked={accessInventory} onChange={e => setAccessInventory(e.target.checked)} disabled={isAdmin} />
              Inventory & Stock
            </label>
          </div>

          <h4 style={{ borderBottom: '1px solid #eee', paddingBottom: '5px', color: '#c0392b' }}>Superuser</h4>
          <label style={{ display: 'flex', alignItems: 'center', gap: '10px', cursor: 'pointer', color: '#c0392b' }}>
            <input type="checkbox" checked={isAdmin} onChange={e => setIsAdmin(e.target.checked)} />
            <strong>Administrator (Bypasses all app restrictions)</strong>
          </label>

          <button type="submit" style={{ width: '100%', padding: '10px', background: '#3498db', color: 'white', border: 'none', cursor: 'pointer', marginTop: '20px', fontWeight: 'bold' }}>
            Save User
          </button>
        </form>

        {/* USERS TABLE */}
        <div style={{ flex: '2', minWidth: '400px', background: '#fff', padding: '20px', borderRadius: '5px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
          <h3>Active Employees</h3>
          <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
            <thead>
              <tr style={{ borderBottom: '2px solid #ccc' }}>
                <th style={{ padding: '8px' }}>ID</th>
                <th style={{ padding: '8px' }}>Username</th>
                <th style={{ padding: '8px' }}>Ticked Apps</th>
              </tr>
            </thead>
            <tbody>
              {userList.map((u) => (
                <tr key={u.id} style={{ borderBottom: '1px solid #eee' }}>
                  <td style={{ padding: '8px' }}>{u.id}</td>
                  <td style={{ padding: '8px' }}><strong>{u.username}</strong></td>
                  <td style={{ padding: '8px' }}>
                    {u.role === 'admin' ? (
                      <span style={{ background: '#e74c3c', color: 'white', padding: '2px 6px', borderRadius: '10px', fontSize: '12px' }}>Admin</span>
                    ) : (
                      <div style={{ display: 'flex', gap: '5px', flexWrap: 'wrap' }}>
                        {u.role.split(',').map(app => (
                           app && <span key={app} style={{ background: '#3498db', color: 'white', padding: '2px 6px', borderRadius: '10px', fontSize: '12px' }}>{app.toUpperCase()}</span>
                        ))}
                      </div>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}