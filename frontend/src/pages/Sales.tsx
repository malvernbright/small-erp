import { useState, useEffect } from 'react';
import api from '../api';

interface Customer {
  id: number;
  name: string;
}

interface Product {
  id: number;
  sku: string;
  name: string;
  stock: number;
  price: string;
}

interface OrderLine {
  productId: number | '';
  quantity: number;
}

export default function Sales() {
  // Master data for dropdowns
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  
  // Form State
  const [customerId, setCustomerId] = useState<number | ''>('');
  const [orderLines, setOrderLines] = useState<OrderLine[]>([{ productId: '', quantity: 1 }]);
  
  // Feedback State
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  // Fetch Customers and Products on mount
  useEffect(() => {
    let isMounted = true;
    
    const fetchData = async () => {
      try {
        const [crmRes, invRes] = await Promise.all([
          api.get('/crm'),
          api.get('/inventory')
        ]);
        
        if (isMounted) {
          setCustomers(crmRes.data);
          setProducts(invRes.data);
        }
      } catch (err) {
        if (isMounted) setError('Failed to load customers or inventory. ' + (err as Error).toString());
      }
    };
    
    fetchData();
    return () => { isMounted = false; };
  }, []);

  // Handlers for the dynamic order lines
  const addOrderLine = () => {
    setOrderLines([...orderLines, { productId: '', quantity: 1 }]);
  };

  const updateOrderLine = (index: number, field: keyof OrderLine, value: number | '') => {
    const newLines = [...orderLines];
    newLines[index] = { ...newLines[index], [field]: value };
    setOrderLines(newLines);
  };

  const removeOrderLine = (index: number) => {
    const newLines = orderLines.filter((_, i) => i !== index);
    setOrderLines(newLines);
  };

  const handlePlaceOrder = async (e: React.FormEvent) => {
    e.preventDefault();
    setMessage('');
    setError('');

    if (customerId === '') {
      setError('Please select a customer.');
      return;
    }

    // Format the payload to match the Rust backend expectations
    const items = orderLines
      .filter(line => line.productId !== '') // Ignore empty lines
      .map(line => ({
        product_id: line.productId,
        quantity: line.quantity
      }));

    if (items.length === 0) {
      setError('Please add at least one product to the order.');
      return;
    }

    try {
      const response = await api.post('/sales', {
        customer_id: customerId,
        items: items
      });
      
      // The backend calculates the grand total safely
      setMessage(`Order #${response.data.order_id} placed successfully! Grand Total: $${response.data.total_amount}`);
      
      // Reset form
      setCustomerId('');
      setOrderLines([{ productId: '', quantity: 1 }]);
    } catch (err: unknown) {
      const errorMessage = (err as { response?: { data: string } }).response?.data || 'Failed to place order.';
      setError(errorMessage);
    }
  };

  return (
    <div style={{ maxWidth: '800px' }}>
      <h2>Create Sales Order</h2>
      <p style={{ color: '#666', fontSize: '14px', marginBottom: '20px' }}>
        Select a customer and add products to the cart. Stock will be automatically deducted upon submission.
      </p>

      {message && <p style={{ color: 'green', padding: '15px', background: '#e6ffe6', borderRadius: '5px' }}><strong>Success:</strong> {message}</p>}
      {error && <p style={{ color: 'red', padding: '15px', background: '#ffe6e6', borderRadius: '5px' }}><strong>Error:</strong> {error}</p>}

      <form onSubmit={handlePlaceOrder} style={{ background: '#fff', padding: '25px', borderRadius: '5px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
        
        {/* Customer Selection */}
        <div style={{ marginBottom: '20px' }}>
          <label style={{ display: 'block', fontWeight: 'bold', marginBottom: '8px' }}>Customer</label>
          <select 
            value={customerId} 
            onChange={(e) => setCustomerId(e.target.value === '' ? '' : Number(e.target.value))}
            style={{ width: '100%', padding: '10px', fontSize: '14px' }}
          >
            <option value="">-- Select a Customer --</option>
            {customers.map(c => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
        </div>

        {/* Dynamic Order Lines */}
        <div style={{ marginBottom: '20px' }}>
          <label style={{ display: 'block', fontWeight: 'bold', marginBottom: '8px' }}>Order Items</label>
          
          {orderLines.map((line, index) => (
            <div key={index} style={{ display: 'flex', gap: '10px', marginBottom: '10px', alignItems: 'center' }}>
              <select 
                value={line.productId}
                onChange={(e) => updateOrderLine(index, 'productId', e.target.value === '' ? '' : Number(e.target.value))}
                style={{ flex: 1, padding: '10px' }}
                required
              >
                <option value="">-- Select Product --</option>
                {products.map(p => (
                  <option key={p.id} value={p.id}>
                    [{p.sku}] {p.name} - ${p.price} ({p.stock} in stock)
                  </option>
                ))}
              </select>
              
              <input 
                type="number" 
                min="1"
                value={line.quantity} 
                onChange={(e) => updateOrderLine(index, 'quantity', Number(e.target.value))}
                style={{ width: '80px', padding: '10px' }} 
                required 
              />
              
              <button 
                type="button" 
                onClick={() => removeOrderLine(index)}
                style={{ padding: '10px', background: '#e74c3c', color: 'white', border: 'none', cursor: 'pointer' }}
                disabled={orderLines.length === 1} // Prevent deleting the last line
              >
                ✕
              </button>
            </div>
          ))}
          
          <button 
            type="button" 
            onClick={addOrderLine}
            style={{ marginTop: '10px', padding: '8px 12px', background: '#ecf0f1', color: '#2c3e50', border: '1px solid #bdc3c7', cursor: 'pointer' }}
          >
            + Add Another Item
          </button>
        </div>

        <button type="submit" style={{ width: '100%', padding: '15px', background: '#27ae60', color: 'white', border: 'none', fontSize: '16px', fontWeight: 'bold', cursor: 'pointer' }}>
          Confirm Sales Order
        </button>
      </form>
    </div>
  );
}