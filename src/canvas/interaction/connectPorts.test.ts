import { describe, expect, it } from 'vitest'

import { canConnectPorts, portFromHit } from './connectPorts'

describe('connectPorts', () => {
  it('parses port hits with handle_id', () => {
    const port = portFromHit({
      id: 'dev:L-0-in-1',
      bounds: { x: 0, y: 0, width: 10, height: 9 },
      node_id: 'dev',
      handle_id: 'L-0-in-1',
    })
    expect(port).toEqual({ nodeId: 'dev', handleId: 'L-0-in-1' })
  })

  it('ignores body hits without handle_id', () => {
    expect(portFromHit({ id: 'dev', bounds: { x: 0, y: 0, width: 10, height: 10 }, node_id: 'dev' })).toBeNull()
  })

  it('rejects connecting a port to itself', () => {
    const port = { nodeId: 'dev', handleId: 'L-0-in-1' }
    expect(canConnectPorts(port, port)).toBe(false)
    expect(canConnectPorts(port, { nodeId: 'other', handleId: 'L-0-in-1' })).toBe(true)
  })
})
