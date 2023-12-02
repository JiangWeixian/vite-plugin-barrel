import type { Plugin } from 'vite'

const name = 'vite-plugin-barrel'

/**
 * Vite plugin development docs
 * @see {@link https://vitejs.dev/guide/api-plugin.html}
 * Rollup lifetime hooks
 * @see {@link https://github.com/neo-hack/rollup-plugin-template/blob/master/src/index.ts}
 */
export const barrel = (): Plugin => {
  return {
    name,
    // https://vitejs.dev/guide/api-plugin.html#vite-specific-hooks
    config() {},
    configResolved() {},
    configureServer() {},
    transformIndexHtml() {},
    handleHotUpdate() {},
    enforce: 'pre',
  }
}
