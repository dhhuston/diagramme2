import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type DragEvent,
  type ReactNode,
} from 'react'
import './LeftPalette.css'

const PALETTE_EXPANDED_KEY = 'diagramme.palette.expanded'
const SECTION_ORDER_KEY = 'diagramme.palette.sectionOrder'
const SECTION_DRAG_TYPE = 'text/plain'

type PaletteSectionId =
  | 'devices'
  | 'av'
  | 'infrastructure'
  | 'annotation'

const DEFAULT_SECTION_ORDER: PaletteSectionId[] = [
  'devices',
  'av',
  'infrastructure',
  'annotation',
]

const SECTION_LABELS: Record<PaletteSectionId, string> = {
  devices: 'Devices',
  av: 'AV',
  infrastructure: 'Infrastructure',
  annotation: 'Annotation',
}

function loadSectionOrder(): PaletteSectionId[] {
  try {
    const raw = localStorage.getItem(SECTION_ORDER_KEY)
    if (!raw) return [...DEFAULT_SECTION_ORDER]
    const parsed = JSON.parse(raw) as unknown
    return normalizeSectionOrder(parsed)
  } catch {
    return [...DEFAULT_SECTION_ORDER]
  }
}

function normalizeSectionOrder(saved: unknown): PaletteSectionId[] {
  const allowed = new Set<PaletteSectionId>(DEFAULT_SECTION_ORDER)
  const picked: PaletteSectionId[] = []
  if (Array.isArray(saved)) {
    for (const x of saved) {
      if (typeof x === 'string' && allowed.has(x as PaletteSectionId)) {
        picked.push(x as PaletteSectionId)
        allowed.delete(x as PaletteSectionId)
      }
    }
  }
  for (const id of DEFAULT_SECTION_ORDER) {
    if (allowed.has(id)) picked.push(id)
  }
  return picked
}

function reorderSections(
  order: PaletteSectionId[],
  dragId: PaletteSectionId,
  dropId: PaletteSectionId
): PaletteSectionId[] | null {
  if (dragId === dropId) return null
  const from = order.indexOf(dragId)
  const to = order.indexOf(dropId)
  if (from < 0 || to < 0) return null
  const next = [...order]
  const [removed] = next.splice(from, 1)
  next.splice(to, 0, removed)
  return next
}

function loadExpanded(): boolean {
  try {
    return localStorage.getItem(PALETTE_EXPANDED_KEY) === 'true'
  } catch {
    return false
  }
}

export type PaletteNodeActions = {
  onAddDevice: () => void
  onAddAvPlate: () => void
  onAddMicBlock: () => void
  onAddSpeakerBlock: () => void
  onAddVolumeControl: () => void
  onAddAntennaSymbol: () => void
  onAddLppPatchPanel: () => void
  onAddDppPatchPanel: () => void
  onAddMlpPatchPanel: () => void
  onAddVpbPatchPanel: () => void
  onAddTextBlock: () => void
  onAddFlyoffNote: () => void
  onAddWiretagPair: () => void
  onAddGroupingZone: () => void
}

export type PaletteViewActions = {
}

export type PaletteProps = {
  nodeActions: PaletteNodeActions
  /** When true, palette insert buttons are disabled (shell-only). */
  disabled?: boolean
}

// ─── Icons ────────────────────────────────────────────────────────────────

function IconPin({ pinned }: { pinned: boolean }) {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      {pinned ? (
        /* Chevrons pointing left — collapse */
        <path d="M10 3L5 8L10 13M13 3L8 8L13 13" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      ) : (
        /* Chevrons pointing right — expand */
        <path d="M6 3L11 8L6 13M3 3L8 8L3 13" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      )}
    </svg>
  )
}

function IconDevice() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <rect x="2" y="2" width="12" height="16" stroke="currentColor" strokeWidth="1.35" />
      <path d="M15 7v6M12 10h6" stroke="currentColor" strokeWidth="1.35" strokeLinecap="round" />
    </svg>
  )
}

function IconAvPlate() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <rect x="3" y="2" width="9" height="16" stroke="currentColor" strokeWidth="1.35" />
      <circle cx="7.5" cy="7" r="1.5" stroke="currentColor" strokeWidth="1.2" />
      <circle cx="7.5" cy="13" r="1.5" stroke="currentColor" strokeWidth="1.2" />
      <path d="M13 6h4M13 10h4M13 14h4" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
    </svg>
  )
}

function IconMic() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <rect x="2" y="5" width="6" height="8" rx="3" stroke="currentColor" strokeWidth="1.35" />
      <path d="M5 13v3M3 16h4" stroke="currentColor" strokeWidth="1.35" strokeLinecap="round" />
      <path d="M1 9.5h1.5M13 9.5h1.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
      <circle cx="15.5" cy="9.5" r="3" stroke="currentColor" strokeWidth="1.2" />
      <path d="M18.5 9.5H20" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
    </svg>
  )
}

function IconSpeaker() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M1 10h3" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
      <rect x="4" y="6.5" width="4" height="7" stroke="currentColor" strokeWidth="1.2" />
      <path d="M8 6.5 16 4 16 16 8 13.5z" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="miter" />
    </svg>
  )
}

function IconVolumeControl() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M1 10h2.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
      <path
        d="M3.5 10 6.5 5.5 12.5 5.5 15.5 10 12.5 14.5 6.5 14.5z"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinejoin="miter"
        fill="none"
      />
      <path d="M15.5 10H19" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
      <text x="10" y="10.5" textAnchor="middle" dominantBaseline="middle" fontSize="5" fontWeight="700" fill="currentColor" stroke="none">
        VC
      </text>
    </svg>
  )
}

function IconAntenna() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M10 4v12" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
      <path d="M6.5 4 10 8.5 13.5 4" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="miter" />
    </svg>
  )
}

function IconPatchPanel({ label }: { label: string }) {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <rect x="2" y="2" width="16" height="16" stroke="currentColor" strokeWidth="1.2" />
      <path d="M5 6.5h2.5M5 10.5h2.5M5 14.5h2.5" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
      <path d="M11.5 6.5l1.5 1.5-1.5 1.5M11.5 10.5l1.5 1.5-1.5 1.5M11.5 14.5l1.5 1.5-1.5 1.5" stroke="currentColor" strokeWidth="1.0" fill="none" />
      {label && (
        <text x="9" y="10" textAnchor="middle" dominantBaseline="middle" fontSize="4.5" fontWeight="700" fill="currentColor" stroke="none">
          {label}
        </text>
      )}
    </svg>
  )
}

function IconText() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <rect x="2" y="3" width="16" height="14" stroke="currentColor" strokeWidth="1.2" />
      <path d="M5 7.5h10M5 10.5h10M5 13.5h7" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
    </svg>
  )
}

function IconFlyoffNote() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M3 5h7v4H3V5z" stroke="currentColor" strokeWidth="1.1" />
      <path d="M12 7h5" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
      <path d="M12 4l4 3-4 3z" fill="currentColor" stroke="none" />
    </svg>
  )
}

/** Two mirrored wire tags (palette icon). */
function IconWiretagPair() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <path d="M2 6 4 8 2 10V6zM5 6h7v4H5V6z" stroke="currentColor" strokeWidth="1.1" strokeLinejoin="miter" />
      <path d="M18 10 16 8 18 6v4zM13 10H6V6h7v4z" stroke="currentColor" strokeWidth="1.1" strokeLinejoin="miter" />
    </svg>
  )
}

function IconGroupingZone() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" aria-hidden="true">
      <rect x="2" y="4" width="16" height="13" stroke="currentColor" strokeWidth="1.2" strokeDasharray="4 2.5" rx="0" />
      <path d="M5 4V2.5" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
      <text x="5" y="3.5" fontSize="5" fontWeight="700" fill="currentColor" stroke="none" dominantBaseline="middle">Zone</text>
    </svg>
  )
}

/** Drag handle for reorderable palette sections (expanded palette only). */
function IconSectionGrip() {
  return (
    <svg width="10" height="14" viewBox="0 0 10 14" fill="none" aria-hidden="true">
      <circle cx="3" cy="3.5" r="1.15" fill="currentColor" />
      <circle cx="7" cy="3.5" r="1.15" fill="currentColor" />
      <circle cx="3" cy="7" r="1.15" fill="currentColor" />
      <circle cx="7" cy="7" r="1.15" fill="currentColor" />
      <circle cx="3" cy="10.5" r="1.15" fill="currentColor" />
      <circle cx="7" cy="10.5" r="1.15" fill="currentColor" />
    </svg>
  )
}

// ─── Tooltip ──────────────────────────────────────────────────────────────

type TooltipState = { label: string; shortcut?: string; y: number } | null

function Tooltip({ state, expanded }: { state: TooltipState; expanded: boolean }) {
  if (!state || expanded) return null
  return (
    <div
      className={`left-palette__tooltip${state ? ' left-palette__tooltip--visible' : ''}`}
      style={{ top: state.y }}
      aria-hidden="true"
    >
      {state.label}
      {state.shortcut && <kbd>{state.shortcut}</kbd>}
    </div>
  )
}

// ─── PaletteButton ────────────────────────────────────────────────────────

type BtnProps = {
  icon: ReactNode
  label: string
  shortcut?: string
  active?: boolean
  disabled?: boolean
  onClick: () => void
  onTooltip: (state: TooltipState) => void
}

function PaletteButton({ icon, label, shortcut, active, disabled, onClick, onTooltip }: BtnProps) {
  const ref = useRef<HTMLButtonElement>(null)

  const showTooltip = useCallback(() => {
    const el = ref.current
    if (!el) return
    const r = el.getBoundingClientRect()
    onTooltip({ label, shortcut, y: r.top + r.height / 2 - 11 })
  }, [label, shortcut, onTooltip])

  const hideTooltip = useCallback(() => {
    onTooltip(null)
  }, [onTooltip])

  return (
    <button
      ref={ref}
      type="button"
      className={`left-palette__btn${active ? ' left-palette__btn--active' : ''}`}
      disabled={disabled}
      onClick={onClick}
      onMouseEnter={showTooltip}
      onMouseLeave={hideTooltip}
      onFocus={showTooltip}
      onBlur={hideTooltip}
      aria-label={label}
      aria-pressed={active !== undefined ? active : undefined}
    >
      {icon}
      <span className="left-palette__btn-label">{label}</span>
    </button>
  )
}

// ─── Section ──────────────────────────────────────────────────────────────

type SectionProps = {
  id: PaletteSectionId
  label: string
  expanded: boolean
  draggingSectionId: PaletteSectionId | null
  children: ReactNode
  onSectionDragStart: (id: PaletteSectionId) => void
  onSectionDragEnd: () => void
  onSectionDragOver: (e: DragEvent<HTMLElement>) => void
  onSectionDrop: (e: DragEvent<HTMLElement>, dropId: PaletteSectionId) => void
}

function Section({
  id,
  label,
  expanded,
  draggingSectionId,
  children,
  onSectionDragStart,
  onSectionDragEnd,
  onSectionDragOver,
  onSectionDrop,
}: SectionProps) {
  const isDragging = draggingSectionId === id

  return (
    <div
      className={`left-palette__section${isDragging ? ' left-palette__section--dragging' : ''}`}
      onDragOver={onSectionDragOver}
      onDrop={(e) => onSectionDrop(e, id)}
    >
      <div className="left-palette__section-head">
        {expanded ? (
          <button
            type="button"
            className="left-palette__section-grip"
            draggable
            aria-label={`Reorder section: ${label}`}
            title="Drag to reorder section"
            onDragStart={(e) => {
              e.dataTransfer.setData(SECTION_DRAG_TYPE, id)
              e.dataTransfer.effectAllowed = 'move'
              onSectionDragStart(id)
            }}
            onDragEnd={onSectionDragEnd}
          >
            <IconSectionGrip />
          </button>
        ) : null}
        <span className="left-palette__section-label">
          <span>{label}</span>
        </span>
      </div>
      <div className="left-palette__section-grid">{children}</div>
    </div>
  )
}

// ─── LeftPalette ──────────────────────────────────────────────────────────

export function LeftPalette({
  nodeActions,
  disabled = false,
}: PaletteProps) {
  const {
    onAddAntennaSymbol,
    onAddAvPlate,
    onAddDevice,
    onAddDppPatchPanel,
    onAddFlyoffNote,
    onAddGroupingZone,
    onAddLppPatchPanel,
    onAddMicBlock,
    onAddMlpPatchPanel,
    onAddSpeakerBlock,
    onAddTextBlock,
    onAddVolumeControl,
    onAddVpbPatchPanel,
    onAddWiretagPair,
  } = nodeActions
  const [expanded, setExpanded] = useState(loadExpanded)
  const [sectionOrder, setSectionOrder] = useState<PaletteSectionId[]>(loadSectionOrder)
  const [draggingSectionId, setDraggingSectionId] = useState<PaletteSectionId | null>(null)
  const [tooltip, setTooltip] = useState<TooltipState>(null)

  const persistSectionOrder = useCallback((order: PaletteSectionId[]) => {
    try {
      localStorage.setItem(SECTION_ORDER_KEY, JSON.stringify(order))
    } catch {
      /* ignore */
    }
  }, [])

  const onSectionDragStart = useCallback((id: PaletteSectionId) => {
    setDraggingSectionId(id)
  }, [])

  const onSectionDragEnd = useCallback(() => {
    setDraggingSectionId(null)
  }, [])

  const onSectionDragOver = useCallback((e: DragEvent<HTMLElement>) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
  }, [])

  const onSectionDrop = useCallback(
    (e: DragEvent<HTMLElement>, dropId: PaletteSectionId) => {
      e.preventDefault()
      const raw = e.dataTransfer.getData(SECTION_DRAG_TYPE)
      if (!raw) return
      const dragId = raw as PaletteSectionId
      if (!DEFAULT_SECTION_ORDER.includes(dragId)) return
      setSectionOrder((prev) => {
        const next = reorderSections(prev, dragId, dropId)
        if (!next) return prev
        persistSectionOrder(next)
        return next
      })
      setDraggingSectionId(null)
    },
    [persistSectionOrder]
  )

  const toggleExpanded = useCallback(() => {
    setExpanded((v) => {
      const next = !v
      try { localStorage.setItem(PALETTE_EXPANDED_KEY, String(next)) } catch { /* */ }
      return next
    })
  }, [])

  function renderPaletteSectionBody(id: PaletteSectionId): ReactNode {
    switch (id) {
      case 'devices':
        return (
          <>
            <PaletteButton icon={<IconDevice />} label="Device block" shortcut="D" onClick={onAddDevice} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconAvPlate />} label="AV plate" shortcut="A" onClick={onAddAvPlate} onTooltip={setTooltip} disabled={disabled} />
          </>
        )
      case 'av':
        return (
          <>
            <PaletteButton icon={<IconMic />} label="Microphone" shortcut="M" onClick={onAddMicBlock} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconSpeaker />} label="Loudspeaker" shortcut="K" onClick={onAddSpeakerBlock} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconVolumeControl />} label="Volume control" shortcut="V" onClick={onAddVolumeControl} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconAntenna />} label="Antenna" shortcut="N" onClick={onAddAntennaSymbol} onTooltip={setTooltip} disabled={disabled} />
          </>
        )
      case 'infrastructure':
        return (
          <>
            <PaletteButton icon={<IconPatchPanel label="L" />} label="Loudspeaker patch" shortcut="1" onClick={onAddLppPatchPanel} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconPatchPanel label="D" />} label="Data patch" shortcut="2" onClick={onAddDppPatchPanel} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconPatchPanel label="M" />} label="Mic/line patch" shortcut="3" onClick={onAddMlpPatchPanel} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconPatchPanel label="V" />} label="Video patch" shortcut="4" onClick={onAddVpbPatchPanel} onTooltip={setTooltip} disabled={disabled} />
          </>
        )
      case 'annotation':
        return (
          <>
            <PaletteButton icon={<IconText />} label="Text block" shortcut="T" onClick={onAddTextBlock} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconFlyoffNote />} label="Flyoff note" shortcut="Y" onClick={onAddFlyoffNote} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconWiretagPair />} label="Wire tag pair" onClick={onAddWiretagPair} onTooltip={setTooltip} disabled={disabled} />
            <PaletteButton icon={<IconGroupingZone />} label="Grouping zone" onClick={onAddGroupingZone} onTooltip={setTooltip} disabled={disabled} />
          </>
        )
      default:
        return null
    }
  }

  // Clear tooltip when palette expands (labels become visible)
  useEffect(() => {
    if (expanded) setTooltip(null)
  }, [expanded])

  const cls = `left-palette${expanded ? ' left-palette--expanded' : ''}`

  return (
    <aside className={cls} aria-label="Palette">
      <button
        type="button"
        className="left-palette__pin"
        onClick={toggleExpanded}
        aria-label={expanded ? 'Collapse palette' : 'Expand palette'}
        title={expanded ? 'Collapse palette' : 'Expand palette'}
      >
        <IconPin pinned={expanded} />
      </button>

      <div className="left-palette__scroll">
        {sectionOrder.map((sectionId) => (
          <Section
            key={sectionId}
            id={sectionId}
            label={SECTION_LABELS[sectionId]}
            expanded={expanded}
            draggingSectionId={draggingSectionId}
            onSectionDragStart={onSectionDragStart}
            onSectionDragEnd={onSectionDragEnd}
            onSectionDragOver={onSectionDragOver}
            onSectionDrop={onSectionDrop}
          >
            {renderPaletteSectionBody(sectionId)}
          </Section>
        ))}
      </div>

      <Tooltip state={tooltip} expanded={expanded} />
    </aside>
  )
}
