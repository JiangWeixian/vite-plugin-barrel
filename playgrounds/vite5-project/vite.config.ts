import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { barrel } from 'vite-plugin-barrel'

const ENABLE_BARREL = !!process.env.ENABLE_BARREL

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    ENABLE_BARREL && barrel({ packages: ['@mui/material', '@mui/icons-material'] }),
  ],
})
