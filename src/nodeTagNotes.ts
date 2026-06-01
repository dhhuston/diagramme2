/**
 * Optional report-only notes on any node (`node.data.tagNotes`).
 * Shown in device / plate tag reports; not rendered on the canvas.
 */

export type NodeTagNotesFields = {
  tagNotes?: string
}

export function readTagNotes(data: unknown): string {
  if (typeof data !== 'object' || data === null) return ''
  const v = (data as Record<string, unknown>).tagNotes
  return typeof v === 'string' ? v.trim() : ''
}
