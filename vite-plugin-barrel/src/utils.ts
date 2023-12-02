import { getQuery } from 'ufo'

import { POST_FIX_RE } from './constants'

export function cleanUrl(url: string): string {
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
