import { useState, useEffect } from 'react';
import api from '../api';

interface Customer {
  id: number;
  name: string;
  email: string | null;
  phone: string | null;
  address: string | null;
}

export default function Crm() {
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [error, setError] = useState('');
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  
  // Form State
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [phone, setPhone] = useState('');
  const [address, setAddress] = useState('');

  useEffect(() => {
    let isMounted = true;
    const loadCustomers = async () => {
      try {
        const response = await api.get('/crm');
        if (isMounted) setCustomers(response.data);
      } catch (err: unknown) {
        if (isMounted) {
          const errorMessage = (err as { response?: { data: string } }).response?.data || 'Failed to fetch customers.';
          setError(errorMessage);
        }
      }
    };
    loadCustomers();
    return () => { isMounted = false; };
  }, [refreshTrigger]);

  const handleAddCustomer = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.post('/crm', {
        name,
        email: email || null, // Convert empty strings to null for the DB
        phone: phone || null,
        address: address || null,
      });
      setRefreshTrigger(prev => prev + 1);
      setName('');
      setEmail('');
      setPhone('');
      setAddress('');
      setError(''); // clear previous errors
    } catch (err: unknown) {
      const errorMessage = (err as { response?: { data: string } }).response?.data || 'Failed to create customer.';
      setError(errorMessage);
    }
  };

  return (
    <div>
      <h2>Customer Relationship Management</h2>
      {error && <p style={{ color: 'red' }}>{error}</p>}

      <div style={{ background: '#f4f4f4', padding: '15px', marginBottom: '20px', borderRadius: '5px' }}>
        <h3>Add New Customer</h3>
        <form onSubmit={handleAddCustomer} style={{ display: 'flex', gap: '10px', alignItems: 'flex-end', flexWrap: 'wrap' }}>
          <div>
            <label style={{ display: 'block', fontSize: '12px' }}>Company/Name *</label>
            <input type="text" value={name} onChange={e => setName(e.target.value)} required />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px' }}>Email</label>
            <input type="email" value={email} onChange={e => setEmail(e.target.value)} />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px' }}>Phone</label>
            <input type="text" value={phone} onChange={e => setPhone(e.target.value)} />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px' }}>Address</label>
            <input type="text" value={address} onChange={e => setAddress(e.target.value)} />
          </div>
          <button type="submit" style={{ padding: '5px 15px', background: '#007bff', color: 'white', border: 'none', cursor: 'pointer' }}>
            Save Customer
          </button>
        </form>
      </div>

      <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
        <thead>
          <tr style={{ borderBottom: '2px solid #ccc' }}>
            <th style={{ padding: '8px' }}>ID</th>
            <th style={{ padding: '8px' }}>Name</th>
            <th style={{ padding: '8px' }}>Email</th>
            <th style={{ padding: '8px' }}>Phone</th>
          </tr>
        </thead>
        <tbody>
          {customers.map((c) => (
            <tr key={c.id} style={{ borderBottom: '1px solid #eee' }}>
              <td style={{ padding: '8px' }}>{c.id}</td>
              <td style={{ padding: '8px' }}><strong>{c.name}</strong></td>
              <td style={{ padding: '8px' }}>{c.email || '—'}</td>
              <td style={{ padding: '8px' }}>{c.phone || '—'}</td>
            </tr>
          ))}
          {customers.length === 0 && (
            <tr><td colSpan={4} style={{ textAlign: 'center', padding: '20px' }}>No customers found.</td></tr>
          )}
        </tbody>
      </table>
    </div>
  );
}