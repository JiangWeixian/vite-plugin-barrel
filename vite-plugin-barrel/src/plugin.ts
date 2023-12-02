import fs from 'node:fs'
import { createRequire } from 'node:module'
import path from 'node:path'

import { transform } from '@swc/core'

import { NODE_MODULES_RE, SCRIPT_RE } from './constants'
import { cleanUrl, parseUrl } from './utils'

import type { Options as SwcConfig } from '@swc/core'
import type { Plugin, PluginContainer } from 'vite'

const name = 'vite-plugin-barrel'

const require = createRequire(import.meta.url)

interface SwcPluginBarrelOptions {
  enable_plugin_relative_import_transform?: boolean
  // TODO: remove this option
  enable_plugin_barrel?: boolean
  wildcard?: boolean
  packages?: string[]
}

const packages = ['@mui/material']

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
      experimental: {
        plugins: [
          [
            require.resolve('swc-plugin-barrel'),
            resolvedPluginOptions,
          ],
        ],
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

const getMappings = async (resourcePath: string, resolve: PluginContainer['resolveId'], name: 'client' | 'server') => {
  const transformMappingCache = barrelTransformMappingCache[name]
  if (transformMappingCache.has(resourcePath)) {
    return transformMappingCache.get(resourcePath)!
  }
  const visited = new Set<string>()
  const getMatches = async (file: string, isWildcard: boolean) => {
    file = cleanUrl(file)
    if (visited.has(file)) {
      return null
    }
    visited.add(file)
    const source = await new Promise<string>((_resolve, reject) => {
      fs.readFile(file, (err: any, data: any) => {
        if (err || data === undefined) {
          console.log(file)
          reject(err)
        } else {
          _resolve(data.toString())
        }
      })
    })
    const { code: output } = await transformSWC(
      resourcePath,
      source,
      { wildcard: isWildcard, enable_plugin_barrel: true, packages },
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
    if (isWildcard) {
      for (const decl of exportList) {
        decl[1] = decl[1] ? (await resolve(path.dirname(file), decl[1]))?.id ?? file : file
        decl[2] = decl[2] || decl[0]
      }
    }
    if (wildcardExports.length) {
      await Promise.all(
        wildcardExports.map(async (req) => {
          const resolvedIdResult = await resolve(
            path.dirname(file),
            req.replace('__barrel_optimize__?names=__PLACEHOLDER__&resourcePath=', ''),
          )
          const targetPath = resolvedIdResult!.id

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

/**
 * Vite plugin development docs
 * @see {@link https://vitejs.dev/guide/api-plugin.html}
 * Rollup lifetime hooks
 * @see {@link https://rollupjs.org/plugin-development/}
 */
export const barrel = (): Plugin[] => {
  return [
    {
      name: `${name}:transform`,
      async transform(source, id) {
        const resolvedId = cleanUrl(id)
        const shouldTransformBarrel = SCRIPT_RE.test(resolvedId) && !NODE_MODULES_RE.test(resolvedId)
        if (shouldTransformBarrel) {
          const { code } = await transformSWC(
            resolvedId,
            source,
            { enable_plugin_barrel: false, packages },
          )
          return code
        }
        return null
      },
      enforce: 'pre',
    },
    {
      name: `${name}:barrel`,
      resolveId(source, importer, options) {
        if (source.startsWith('__barrel_optimize__')) {
          return source
        }
        return null
      },
      async load(id, options) {
        if (id.startsWith('__barrel_optimize__')) {
          console.log(id)
          const params = parseUrl(id)
          const resolve = this.resolve.bind(this)
          let resourcePath = params.resourcePath
          try {
            resourcePath = (await resolve(params.resourcePath))!.id
          } catch (e) {
            console.log(resourcePath)
          }
          const mapping = await getMappings(cleanUrl(resourcePath), resolve, 'client')
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

          console.log(output)
          return output
        }
        return null
      },
      enforce: 'pre',
    },
  ]
}
