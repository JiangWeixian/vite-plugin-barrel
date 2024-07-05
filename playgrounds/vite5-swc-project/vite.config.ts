import react from '@vitejs/plugin-react-swc'
import { defineConfig } from 'vite'
import { barrel } from 'vite-plugin-barrel'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    process.env.ENABLE_BARREL && barrel({ packages: ['@mui/material', '@mui/icons-material'] }),
  ],
})
