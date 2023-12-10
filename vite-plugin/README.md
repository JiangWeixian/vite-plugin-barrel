# vite-plugin-barrel
> A vite port version of [next.js optimizePackagesImports](https://nextjs.org/docs/app/api-reference/next-config-js/optimizePackageImports)

[![npm](https://img.shields.io/npm/v/vite-plugin-barrel)](https://github.com/JiangWeixian/vite-plugin-template) [![GitHub](https://img.shields.io/npm/l/vite-plugin-barrel)](https://github.com/JiangWeixian/vite-plugin-template)

## why

Some packages exports lots of modules, it will cause vite transform lots of files in build step. For example, `@mui/icons-material` exports 1000+ components, it's harmful for vite build performance.

![benchmark](https://github.com/JiangWeixian/repo-images/assets/6839576/9737eebb-0722-4d2f-a058-945141951891)

Test on `Apple M1 Pro`, with this plugin, it improve 50%+ build performance.

## install

```console
pnpm add vite-plugin-barrel
```

## usage

```ts
// vite.config.ts
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { barrel } from 'vite-plugin-barrel'

// https://vitejs.dev/config/
export default defineConfig({
  optimizeDeps: {
    exclude: ['@mui/material', '@mui/icons-material'],
    force: true,
  },
  plugins: [
    react(),
    barrel({ packages: ['@mui/material', '@mui/icons-material'] }),
  ],
})
```

## credits

[next.js optimize_barrel](https://nextjs.org/docs/app/api-reference/next-config-js/optimizePackageImports)

# 
<div align='right'>

*built with ❤️ by 😼*

</div>

