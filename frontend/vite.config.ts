import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    // This forces Vite to use a single instance of React across all packages
    dedupe: ['react', 'react-dom'],
  },
})
