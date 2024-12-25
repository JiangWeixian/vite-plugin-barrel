import react from '@vitejs/plugin-react-swc'
import { defineConfig } from 'vite'
import { barrel, swc_plugin_barrel } from 'vite-plugin-barrel'

const ENABLE_BARREL = !!process.env.ENABLE_BARREL

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({ plugins: ENABLE_BARREL ? [swc_plugin_barrel({ packages: ['@mui/material', '@mui/icons-material'] })] : [] }),
    ENABLE_BARREL && barrel({ packages: ['@mui/material', '@mui/icons-material'], experimental: { integration: 'plugin-react-swc' } }),
  ].filter(Boolean),
})
