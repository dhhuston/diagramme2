/** Fallback when `getComputedStyle` is unavailable (tests). */
const GRID_COLOR_FALLBACK = 'oklch(0.9 0.006 155)'
const GRID_MAJOR_FALLBACK = 'oklch(0.84 0.008 155)'
const SELECTION_STROKE_FALLBACK = 'oklch(0.52 0.18 155)'
const SCHEMATIC_FACE_FALLBACK = '#ffffff'

let cachedGridMajorColor: string | null = null

let cachedGridColor: string | null = null
let cachedSelectionStroke: string | null = null
let cachedSchematicFace: string | null = null

/** Resolve a diagram canvas color from CSS custom properties (Konva cannot use `var()`). */
export function resolveCanvasColor(
  varName: string,
  fallback = GRID_COLOR_FALLBACK,
): string {
  if (typeof document === 'undefined') {
    return fallback
  }
  const raw = getComputedStyle(document.documentElement).getPropertyValue(varName).trim()
  return raw || fallback
}

export function getCanvasGridColor(): string {
  if (cachedGridColor == null) {
    cachedGridColor = resolveCanvasColor('--clr-grid')
  }
  return cachedGridColor
}

/** Call after theme / CSS variable changes (optional). */
export function getCanvasGridMajorColor(): string {
  if (cachedGridMajorColor == null) {
    cachedGridMajorColor = resolveCanvasColor('--clr-grid-major', GRID_MAJOR_FALLBACK)
  }
  return cachedGridMajorColor
}

export function getCanvasSchematicFaceColor(): string {
  if (cachedSchematicFace == null) {
    cachedSchematicFace = resolveCanvasColor(
      '--clr-schematic-face',
      SCHEMATIC_FACE_FALLBACK,
    )
  }
  return cachedSchematicFace
}

export function getCanvasSelectionStroke(): string {
  if (cachedSelectionStroke == null) {
    cachedSelectionStroke = resolveCanvasColor('--clr-accent', SELECTION_STROKE_FALLBACK)
  }
  return cachedSelectionStroke
}

export function invalidateCanvasTokenCache(): void {
  cachedGridColor = null
  cachedGridMajorColor = null
  cachedSelectionStroke = null
  cachedSchematicFace = null
}
