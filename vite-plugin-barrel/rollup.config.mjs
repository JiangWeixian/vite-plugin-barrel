import esbuild from 'rollup-plugin-esbuild'
import alias from '@rollup/plugin-alias'
import { externals } from 'rollup-plugin-node-externals'
import resolve from '@rollup/plugin-node-resolve'
import size from 'rollup-plugin-filesize'
import ce from 'rollup-plugin-condition-exports'

import { defineConfig } from 'rollup'

export default defineConfig([
  // CommonJS (for Node) and ES module (for bundlers) build.
  // (We could have three entries in the configuration array
  // instead of two, but it's quicker to generate multiple
  // builds from a single configuration where possible, using
  // an array for the `output` option, where we can specify
  // `file` and `format` for each target)
  {
    plugins: [
      externals({ devDeps: false }),
      esbuild({
        target: 'es2020',
      }), // so Rollup can convert TypeScript to JavaScript
      alias({
        resolve: ['.ts', '.js', '.tsx', '.jsx'],
        entries: [{ find: '@/', replacement: './src/' }],
      }),
      resolve({ preferBuiltins: true }),
      size(),
      ce(),
    ],
    output: [
      { dir: 'dist', entryFileNames: '[name].cjs', format: 'cjs' },
      { dir: 'dist', entryFileNames: '[name].mjs', format: 'es' },
    ],
  },
])
