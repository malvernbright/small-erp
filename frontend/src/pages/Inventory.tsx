import { useState, useEffect } from 'react';
import api from '../api';

interface Product {
  id: number;
  sku: string;
  name: string;
  description: string | null;
  stock: number;
  price: string; 
}

export default function Inventory() {
  const [products, setProducts] = useState<Product[]>([]);
  const [error, setError] = useState('');
  
  // 1. The Trigger State: We update this number to force the useEffect to run again
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  
  const [newSku, setNewSku] = useState('');
  const [newName, setNewName] = useState('');
  const [newStock, setNewStock] = useState(0);
  const [newPrice, setNewPrice] = useState('0.00');

  // 2. The useEffect now completely owns the fetch logic
  useEffect(() => {
    let isMounted = true; // Protects against React StrictMode double-mounting

    const loadInventory = async () => {
      try {
        const response = await api.get('/inventory');
        if (isMounted) setProducts(response.data);
      } catch (err: unknown) {
        if (isMounted) {
          const errorMessage = (err as { response?: { data: string } }).response?.data || 'Failed to fetch inventory.';
          setError(errorMessage);
        }
      }
    };

    loadInventory();

    return () => {
      isMounted = false; // Cleanup function if component unmounts mid-fetch
    };
  }, [refreshTrigger]); // <-- React will re-run the effect anytime this changes

  const handleAddProduct = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await api.post('/inventory', {
        sku: newSku,
        name: newName,
        description: '', 
        stock: Number(newStock),
        price: newPrice, 
      });
      
      // 3. Trigger a table refresh cleanly
      setRefreshTrigger(prev => prev + 1); 
      
      setNewSku('');
      setNewName('');
      setNewStock(0);
      setNewPrice('0.00');
    } catch (err: unknown) {
      const errorMessage = (err as { response?: { data: string } }).response?.data || 'Failed to create product.';
      setError(errorMessage);
    }
  };

  const handleAdjustStock = async (id: number, currentStock: number) => {
    const qtyStr = window.prompt(`Enter amount to add (use negative numbers to remove).\nCurrent stock: ${currentStock}`);
    if (!qtyStr) return;
    
    const qtyChange = parseInt(qtyStr, 10);
    if (isNaN(qtyChange)) {
      alert("Invalid number");
      return;
    }

    try {
      await api.patch(`/inventory/${id}/stock`, {
        quantity_change: qtyChange,
      });
      
      // 4. Trigger a table refresh cleanly
      setRefreshTrigger(prev => prev + 1);
      
    } catch (err: unknown) {
      const errorMessage = (err as { response?: { data: string } }).response?.data || 'Failed to adjust stock.';
      alert(errorMessage);
    }
  };

  return (
    <div>
      <h2>Inventory Management</h2>
      {error && <p style={{ color: 'red' }}>{error}</p>}

      {/* Add Product Form */}
      <div style={{ background: '#f4f4f4', padding: '15px', marginBottom: '20px', borderRadius: '5px' }}>
        <h3>Add New Product</h3>
        <form onSubmit={handleAddProduct} style={{ display: 'flex', gap: '10px', alignItems: 'flex-end' }}>
          <div>
            <label style={{ display: 'block', fontSize: '12px' }}>SKU</label>
            <input type="text" value={newSku} onChange={e => setNewSku(e.target.value)} required />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px' }}>Name</label>
            <input type="text" value={newName} onChange={e => setNewName(e.target.value)} required />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px' }}>Initial Stock</label>
            <input type="number" value={newStock} onChange={e => setNewStock(Number(e.target.value))} required />
          </div>
          <div>
            <label style={{ display: 'block', fontSize: '12px' }}>Price ($)</label>
            <input type="text" step="0.01" value={newPrice} onChange={e => setNewPrice(e.target.value)} required />
          </div>
          <button type="submit" style={{ padding: '5px 15px', background: '#28a745', color: 'white', border: 'none', cursor: 'pointer' }}>
            Add
          </button>
        </form>
      </div>

      {/* Inventory Table */}
      <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
        <thead>
          <tr style={{ borderBottom: '2px solid #ccc' }}>
            <th style={{ padding: '8px' }}>ID</th>
            <th style={{ padding: '8px' }}>SKU</th>
            <th style={{ padding: '8px' }}>Name</th>
            <th style={{ padding: '8px' }}>Stock</th>
            <th style={{ padding: '8px' }}>Price</th>
            <th style={{ padding: '8px' }}>Actions</th>
          </tr>
        </thead>
        <tbody>
          {products.map((p) => (
            <tr key={p.id} style={{ borderBottom: '1px solid #eee' }}>
              <td style={{ padding: '8px' }}>{p.id}</td>
              <td style={{ padding: '8px' }}>{p.sku}</td>
              <td style={{ padding: '8px' }}>{p.name}</td>
              <td style={{ padding: '8px' }}><strong>{p.stock}</strong></td>
              <td style={{ padding: '8px' }}>${p.price}</td>
              <td style={{ padding: '8px' }}>
                <button 
                  onClick={() => handleAdjustStock(p.id, p.stock)}
                  style={{ padding: '4px 8px', cursor: 'pointer' }}
                >
                  Adjust Stock
                </button>
              </td>
            </tr>
          ))}
          {products.length === 0 && (
            <tr><td colSpan={6} style={{ textAlign: 'center', padding: '20px' }}>No products found. Add one above!</td></tr>
          )}
        </tbody>
      </table>
    </div>
  );
}