import path from 'node:path'

import { describe, expect, it } from 'vitest'

import { resolver } from '../src/utils'

describe('resolver', () => {
  it('custom', async () => {
    const root = path.resolve(expect.getState().testPath!, '../../..')
    const playground = path.join(root, 'playgrounds/vite4-project')
    const result = await resolver('@mui/material', playground)
    expect(result).toBeDefined()
  })
})
