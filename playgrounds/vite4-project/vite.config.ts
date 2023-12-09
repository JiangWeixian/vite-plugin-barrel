import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { barrel } from 'vite-plugin-barrel'

// https://vitejs.dev/config/
export default defineConfig({
  optimizeDeps: {
    exclude: ['@mui/material', '@mui/icons-material'],
    force: true,
  },
  plugins: [react(), barrel({ packages: ['@mui/material', '@mui/icons-material'] })],
})
