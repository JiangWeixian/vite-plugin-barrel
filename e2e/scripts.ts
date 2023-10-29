import path from 'node:path'

import { transform } from '@swc/core'

const pluginName = 'swc_plugin_barrel.wasm'
const pluginPath = path.resolve(process.cwd(), './', pluginName)

const main = async () => {
  const content = `
import { Button, ALink } from "foo";

const Card = () => {
  return <Button />
}

import { z } from './3'

export { foo, b as y } from './1'
export { x, a } from './2'
export { z }
  `
  const result = await transform(content, {
    jsc: {
      parser: {
        syntax: 'typescript',
        tsx: true,
      },
      experimental: {
        plugins: [
          [pluginPath, { packages: ['foo'] }],
        ],
      },
    },
  })
  console.log(result)
}

main()
