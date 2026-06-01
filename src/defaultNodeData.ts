/** Default `data` payloads for palette-created nodes (ported from v6). */

function numberedRows(prefix: string, from: number, to: number) {
  return Array.from({ length: to - from + 1 }, (_, i) => {
    const n = from + i
    return { id: `${prefix}-${n}`, label: String(n) }
  })
}

export function createDefaultDeviceData(deviceIndex: number) {
  const slug = crypto.randomUUID().slice(0, 8)
  return {
    tagCode: 'DEV',
    tagNumber: String(deviceIndex),
    description: 'New device',
    leftColumn: [{ header: 'Input', rows: [{ id: `in-${slug}`, label: '1' }] }],
    rightColumn: [{ header: 'Output', rows: [{ id: `out-${slug}`, label: '1' }] }],
  }
}

export function createDefaultAvPlateData(plateIndex: number) {
  const slug = crypto.randomUUID().slice(0, 8)
  return {
    tagCode: 'AVP',
    tagNumber: String(plateIndex),
    description: 'AV PLATE',
    groups: [
      {
        header: 'Group 1',
        rows: [{ id: `p-${slug}-a`, label: '1', direction: 'input' as const }],
      },
    ],
  }
}

export function createDefaultMicBlockData(micIndex: number) {
  return { line1: `MIC ${micIndex}`, line2: 'OVERHEAD', channelNumber: String(micIndex) }
}

export function createDefaultSpeakerBlockData(speakerIndex: number) {
  return {
    line1: `SPK ${speakerIndex}`,
    line2: 'PLAN NORTH HL',
    symbolKind: 'standard',
    passthruEnabled: false,
  }
}

export function createDefaultVolumeControlData() {
  return {}
}

export function createDefaultAntennaSymbolData() {
  return { line1: 'ANT' }
}

export function createDefaultLppPatchPanelData(i: number) {
  const slug = crypto.randomUUID().slice(0, 8)
  return {
    tagCode: 'LPP',
    tagNumber: String(i),
    descriptionLines: ['Loudspeaker patch panel'],
    rows: [
      { id: `lpp-${slug}-a`, connected: true },
      { id: `lpp-${slug}-b`, connected: false },
    ],
  }
}

export function createDefaultDppPatchPanelData(i: number) {
  const slug = crypto.randomUUID().slice(0, 8)
  return {
    tagCode: 'DPP',
    tagNumber: String(i),
    descriptionLines: ['Data patch panel'],
    rows: [
      { id: `dpp-${slug}-a`, label: '1', direction: 'output' as const },
      { id: `dpp-${slug}-b`, label: '2', direction: 'input' as const },
    ],
  }
}

export function createDefaultMlpPatchPanelData(i: number) {
  const slug = crypto.randomUUID().slice(0, 8)
  return {
    tagCode: 'APP',
    tagNumber: String(i),
    descriptionLines: ['Mic/line', 'Patch panel'],
    rows: [
      { id: `mlp-${slug}-a`, normalling: 'HN' as const },
      { id: `mlp-${slug}-b`, normalling: '' as const },
    ],
  }
}

export function createDefaultVpbPatchPanelData(i: number) {
  const slug = crypto.randomUUID().slice(0, 8)
  return {
    tagCode: 'VPP',
    tagNumber: String(i),
    descriptionLines: ['Video', 'Patch panel'],
    rows: [
      { id: `vpb-${slug}-a`, normalling: 'N' as const },
      { id: `vpb-${slug}-b`, normalling: '' as const },
    ],
  }
}

export { numberedRows }
