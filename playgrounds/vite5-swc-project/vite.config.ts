import react from '@vitejs/plugin-react-swc'
import { defineConfig } from 'vite'
import { barrel, swc_plugin_barrel } from 'vite-plugin-barrel'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({ plugins: process.env.ENABLE_BARREL ? [swc_plugin_barrel({ packages: ['@mui/material', '@mui/icons-material'] })] : [] }),
    process.env.ENABLE_BARREL && barrel({ packages: ['@mui/material', '@mui/icons-material'], integration: 'plugin-react-swc' }),
  ],
})
