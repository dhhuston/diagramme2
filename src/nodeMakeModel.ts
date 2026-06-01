/**
 * Optional manufacturer/model/datasheet metadata on any node. Stored in `node.data` for save/load;
 * included in diagram interchange (XLSX / JSON export model), not rendered on DXF.
 */

export type NodeMakeModelFields = {
  manufacturer?: string
  model?: string
  datasheetLink?: string
}

export type NodeEquipmentMeta = {
  manufacturer: string
  model: string
  datasheetLink: string
}

export function readMakeModel(data: unknown): NodeEquipmentMeta {
  if (typeof data !== 'object' || data === null) {
    return { manufacturer: '', model: '', datasheetLink: '' }
  }
  const d = data as Record<string, unknown>
  const manufacturer =
    typeof d.manufacturer === 'string'
      ? d.manufacturer
      : typeof d.make === 'string'
        ? d.make
        : ''
  return {
    manufacturer,
    model: typeof d.model === 'string' ? d.model : '',
    datasheetLink: typeof d.datasheetLink === 'string' ? d.datasheetLink : '',
  }
}

/** Spread into `DiagramExportEntity.properties` (export schema v2+). */
export function makeModelExportProperties(
  data: unknown,
): Record<string, string> {
  const { manufacturer, model, datasheetLink } = readMakeModel(data)
  return { manufacturer, model, datasheetLink }
}
