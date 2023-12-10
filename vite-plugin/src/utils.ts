import EnhancedResolve from 'enhanced-resolve'
import { getQuery } from 'ufo'

import { BARREL_MODULE_RE, POST_FIX_RE } from './constants'

export function cleanUrl(url?: string): string {
  if (!url) {
    return ''
  }
  return url.replace(POST_FIX_RE, '')
}

interface BarrelParams {
  names: string[]
  resourcePath: string
}

export function parseUrl(url: string): BarrelParams {
  const params = getQuery(url) as any
  const names = Array.isArray(params.names) ? params.names : [params.names]
  params.names = names
  return params
}

export const resolver = async (id: string, context: string): Promise<string> => {
  const _resolver = EnhancedResolve.create({
    extensions: ['.mjs', '.js', '.ts', '.tsx'],
    // see more options below,
    mainFields: ['module'],
  })
  return new Promise((resolve) => {
    _resolver(context, id, (_error, result) => {
      resolve(result as string)
    })
  })
}

export const isBarrelModule = (id: string) => {
  return BARREL_MODULE_RE.test(id)
}
