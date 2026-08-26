import axios from 'axios';

// Create an Axios instance pointing to our Rust backend
const api = axios.create({
  baseURL: 'http://127.0.0.1:3000/api',
});

// Intercept requests and attach the JWT token if it exists
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

export default api;