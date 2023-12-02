import react from '@vitejs/plugin-react'
import { barrel } from 'vite-plugin-barrel'
import { defineConfig } from 'vite'

// https://vitejs.dev/config/
export default defineConfig({
  optimizeDeps: {
    exclude: ['@mui/material'],
    force: true,
  },
  plugins: [react(), barrel()],
})
