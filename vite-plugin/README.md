# vite-plugin-barrel
> A vite port version of [next.js optimizePackagesImports](https://nextjs.org/docs/app/api-reference/next-config-js/optimizePackageImports)

[![npm](https://img.shields.io/npm/v/vite-plugin-barrel)](https://github.com/JiangWeixian/vite-plugin-template) [![GitHub](https://img.shields.io/npm/l/vite-plugin-barrel)](https://github.com/JiangWeixian/vite-plugin-template)

## why

Some packages exports lots of modules, it will cause vite transform lots of files in build step. For example, `@mui/icons-material` exports 1000+ components, it's harmful for vite build performance.

![benchmark](https://github.com/JiangWeixian/repo-images/blob/master/barrel/barrel.png?raw=true)

Test on `Apple M1 Pro`, with this plugin, it improve 50%+ build performance.

> [!NOTE]  
> According to [swc_compat_range_docs](https://plugins.swc.rs/versions/range/94), you should select right `@swc/core` version.
> - `vite-plugin-barrel@0.2.x ~ 0.3.x` and `swc-plugin-barrel@0.1.x ~ 0.2.x` use `swc_core@0.96.x` compat with `1.6.x`
> - `vite-plugin-barrel@0.4.x` and `swc-plugin-barrel@0.3.x` use `swc_core@0.105.0` compat with `v1.7.0-v1.7.27`

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
  plugins: [
    react(),
    barrel({ packages: ['@mui/material', '@mui/icons-material'] }),
  ],
})
```

## options

### `options.packages`

- Type: `string[]`

The packages you want to optimize.

### `options.experimental.intergration`

- Type: `plugin-react-swc`
- Optional

vite-plugin-barrel will use `@swc/core` to transform code with `swc-plugin-barrel`. You can pass this plugin directly to `@vitejs/plugin-react-swc` disable extra transform to improve performance.

```ts
// vite.config.ts
import react from '@vitejs/plugin-react-swc'
import { defineConfig } from 'vite'
import { barrel, swc_plugin_barrel } from 'vite-plugin-barrel'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({
      plugins: [
        swc_plugin_barrel({
          packages: ['@mui/material', '@mui/icons-material']
        })
      ]
    }),
    barrel({
      packages: ['@mui/material', '@mui/icons-material'],
      experimental: {
        intergration: 'plugin-react-swc'
      }
    }),
  ],
})
```

## credits

[next.js optimize_barrel](https://nextjs.org/docs/app/api-reference/next-config-js/optimizePackageImports)

#
<div align='right'>

*built with ❤️ by 😼*

</div>
