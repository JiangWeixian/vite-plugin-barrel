import fs from 'node:fs'
import { createRequire } from 'node:module'
import { dirname } from 'node:path'
import { performance } from 'node:perf_hooks'

import { transform } from '@swc/core'

import { NODE_MODULES_RE, SCRIPT_RE } from './constants'
import {
  cleanUrl,
  debug,
  isBarrelModule,
  parseUrl,
  resolver,
} from './utils'

import type { Options as SwcConfig } from '@swc/core'
import type { Plugin } from 'vite'

const name = 'vite-plugin-barrel'

const require = createRequire(import.meta.url)

interface SwcPluginBarrelOptions {
  enable_plugin_relative_import_transform?: boolean
  enable_plugin_barrel?: boolean
  wildcard?: boolean
  packages?: string[]
}

export const swc_plugin_barrel = ({ packages }: Pick<SwcPluginBarrelOptions, 'packages'> = { packages: [] }) => {
  return [
    require.resolve('swc-plugin-barrel'),
    {
      enable_plugin_barrel: false,
      enable_plugin_relative_import_transform: false,
      wildcard: false,
      packages: packages ?? [],
    },
  ]
}

const transformSWC = async (
  resourcePath: string,
  source: string,
  pluginOptions: SwcPluginBarrelOptions = {},
) => {
  const isTypescript = resourcePath.endsWith('.ts') || resourcePath.endsWith('.tsx')
  const resolvedPluginOptions: SwcPluginBarrelOptions = {
    enable_plugin_barrel: pluginOptions.enable_plugin_barrel ?? false,
    enable_plugin_relative_import_transform: pluginOptions.enable_plugin_relative_import_transform ?? false,
    wildcard: pluginOptions.wildcard ?? false,
    packages: pluginOptions.packages ?? [],
  }
  const config: SwcConfig = {
    sourceMaps: false,
    jsc: {
      target: 'es2021',
      experimental: {
        plugins: [
          [
            require.resolve('swc-plugin-barrel'),
            resolvedPluginOptions,
          ],
        ],
      },
      transform: {
        react: {
          runtime: 'automatic',
          refresh: false,
          development: false,
        },
      },
      parser: {
        syntax: isTypescript ? 'typescript' : 'ecmascript',
        [isTypescript ? 'tsx' : 'jsx']: true,
      },
    },
  }
  const result = await transform(source, config)
  return result
}

type TransformCache = Map<
  string,
  {
    prefix: string
    exportList: [string, string, string][]
    wildcardExports: string[]
  } | null
>

const barrelTransformMappingCache: {
  client: TransformCache
  server: TransformCache
} = {
  client: new Map(),
  server: new Map(),
}

const getMappings = async (resourcePath: string, resolve: any, name: 'client' | 'server', options: Options) => {
  const transformMappingCache = barrelTransformMappingCache[name]
  if (transformMappingCache.has(resourcePath)) {
    return transformMappingCache.get(resourcePath)!
  }
  const visited = new Set<string>()
  const getMatches = async (file: string, isWildcard: boolean) => {
    if (visited.has(file)) {
      return null
    }
    visited.add(file)
    const source = await new Promise<string>((_resolve, reject) => {
      fs.readFile(file, (err: any, data: any) => {
        if (err || data === undefined) {
          reject(err)
        } else {
          _resolve(data.toString())
        }
      })
    })
    const { code: output } = await transformSWC(
      resourcePath,
      source,
      { wildcard: isWildcard, enable_plugin_barrel: true, packages: options.packages },
    )
    const matches = output.match(
      /^([\s\S]*)export (const|var) __next_private_exports_map__ = ('[^']+'|"[^"]+")/,
    )
    if (!matches) {
      return null
    }
    const prefix = matches[1]
    let exportList = JSON.parse(matches[3].slice(1, -1)) as [
      string,
      string,
      string,
    ][]
    const wildcardExports = [
      ...output.matchAll(/export \* from "([^"]+)"/g),
    ].map(match => match[1])
    // In the wildcard case, if the value is exported from another file, we
    // redirect to that file (decl[0]). Otherwise, export from the current
    // file itself.
    for (const decl of exportList) {
      decl[1] = decl[1] ? cleanUrl(await resolve(decl[1], dirname(file))) ?? file : file
      decl[2] = decl[2] || decl[0]
    }
    if (wildcardExports.length) {
      await Promise.all(
        wildcardExports.map(async (req) => {
          const resolvedIdResult = await resolve(
            req.replace('__barrel_optimize__?names=__PLACEHOLDER__&resourcePath=', ''),
            dirname(file),
          )
          const targetPath = cleanUrl(resolvedIdResult)

          const targetMatches = await getMatches(targetPath, true)
          if (targetMatches) {
            // Merge the export list
            exportList = exportList.concat(targetMatches.exportList)
          }
        }),
      )
    }
    return {
      prefix,
      exportList,
      wildcardExports,
    }
  }
  const res = await getMatches(resourcePath, false)

  transformMappingCache.set(resourcePath, res)
  return res
}

interface Options extends Pick <SwcPluginBarrelOptions, 'packages'> {
  experimental?: {
    integration?: 'plugin-react-swc'
  }
}

/**
 * Vite plugin development docs
 * @see {@link https://vitejs.dev/guide/api-plugin.html}
 * Rollup lifetime hooks
 * @see {@link https://rollupjs.org/plugin-development/}
 */
export const barrel = ({
  packages = [],
  experimental = {},
}: Options = {
  packages: [],
  experimental: {},
}): Plugin[] => {
  const viteConfig: {
    root: string
  } = {
    root: process.cwd(),
  }
  const benchmark = {
    [`${name}:transform`]: 0,
    [`${name}:barrel`]: 0,
  }
  return [
    {
      name: `${name}:transform`,
      apply: 'build',
      enforce: 'pre',
      async transform(source, id) {
        if (experimental?.integration) {
          return null
        }
        const resolvedId = cleanUrl(id)
        const shouldTransformBarrel = SCRIPT_RE.test(resolvedId) && !NODE_MODULES_RE.test(resolvedId)
        if (shouldTransformBarrel) {
          const now = performance.now()
          const { code, map } = await transformSWC(
            resolvedId,
            source,
            { enable_plugin_barrel: false, packages },
          )
          const took = performance.now() - now
          benchmark[`${name}:transform`] += took
          return {
            code,
            map,
          }
        }
        return null
      },
      buildEnd() {
        debug(`plugin ${`${name}:transform`} took ${benchmark[`${name}:transform`]}ms`)
      },
    },
    {
      name: `${name}:barrel`,
      apply: 'build',
      enforce: 'pre',
      configResolved(config) {
        viteConfig.root = config.root
      },
      async resolveId(source) {
        if (isBarrelModule(source)) {
          return `\0${source}`
        }
        return null
      },
      async load(id, options) {
        if (isBarrelModule(id)) {
          const now = performance.now()
          const params = parseUrl(id)
          const resourcePath = await resolver(params.resourcePath, viteConfig.root)
          debug('load barrel %s', resourcePath)
          const bundler = options?.ssr ? 'server' : 'client'
          const mapping = await getMappings(
            cleanUrl(resourcePath),
            resolver,
            bundler,
            { packages },
          )
          if (!mapping) {
            return `export * from "${resourcePath}"`
          }
          // It needs to keep the prefix for comments and directives like "use client".
          const prefix = mapping.prefix
          const exportList = mapping.exportList
          const exportMap = new Map<string, [string, string]>()
          for (const [name, filePath, orig] of exportList) {
            exportMap.set(name, [filePath, orig])
          }
          let output = prefix
          const missedNames: string[] = []
          for (const name of params.names) {
            // If the name matches
            if (exportMap.has(name)) {
              const decl = exportMap.get(name)!
              if (decl[1] === '*') {
                output += `\nexport * as ${name} from ${JSON.stringify(decl[0])}`
              } else if (decl[1] === 'default') {
                output += `\nexport { default as ${name} } from ${JSON.stringify(
                  decl[0],
                )}`
              } else if (decl[1] === name) {
                output += `\nexport { ${name} } from ${JSON.stringify(decl[0])}`
              } else {
                output += `\nexport { ${decl[1]} as ${name} } from ${JSON.stringify(
                  decl[0],
                )}`
              }
            } else {
              missedNames.push(name)
            }
          }
          // These are from wildcard exports.
          if (missedNames.length > 0) {
            for (const req of mapping.wildcardExports) {
              output += `\nexport * from ${JSON.stringify(
                req.replace('__PLACEHOLDER__', `${missedNames.join(',')}&wildcard`),
              )}`
            }
          }

          debug('optimized barrel output %s', output)
          const took = performance.now() - now
          benchmark[`${name}:barrel`] += took
          return output
        }
        return null
      },
      buildEnd() {
        debug(`plugin ${`${name}:barrel`} took ${benchmark[`${name}:barrel`]}ms`)
      },
    },
  ]
}
