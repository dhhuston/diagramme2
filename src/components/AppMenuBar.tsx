import { useEffect, useMemo, useRef, useState } from 'react'
import './AppMenuBar.css'

export type MenuBarSeparator = { kind: 'separator' }

export type MenuBarItem = {
  kind: 'item'
  id: string
  label: string
  shortcut?: string
  disabled?: boolean
  /** Non-interactive label (mirrors empty submenus without fake actions). */
  inert?: boolean
  onSelect?: () => void
}

export type MenuBarRow = MenuBarSeparator | MenuBarItem

export type AppMenuBarBucket = {
  id: string
  title: string
  rows: MenuBarRow[]
}

export type AppMenuBarProps = {
  menus: AppMenuBarBucket[]
}

export function AppMenuBar({ menus }: AppMenuBarProps) {
  const [openMenu, setOpenMenu] = useState<string | null>(null)
  const rootRef = useRef<HTMLDivElement | null>(null)

  const menuById = useMemo(() => {
    const m = new Map<string, AppMenuBarBucket>()
    for (const bucket of menus) m.set(bucket.id, bucket)
    return m
  }, [menus])

  useEffect(() => {
    const onPointerDown = (event: PointerEvent) => {
      const root = rootRef.current
      if (!root) return
      if (!root.contains(event.target as Node)) setOpenMenu(null)
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpenMenu(null)
    }
    window.addEventListener('pointerdown', onPointerDown)
    window.addEventListener('keydown', onKeyDown)
    return () => {
      window.removeEventListener('pointerdown', onPointerDown)
      window.removeEventListener('keydown', onKeyDown)
    }
  }, [])

  return (
    <div className="app-menu-bar" ref={rootRef} aria-label="Application menu bar">
      {menus.map((bucket) => (
        <div className="app-menu-bar__menu" key={bucket.id}>
          <button
            type="button"
            className={`app-menu-bar__trigger${openMenu === bucket.id ? ' app-menu-bar__trigger--open' : ''}`}
            onClick={() => setOpenMenu((current) => (current === bucket.id ? null : bucket.id))}
            aria-haspopup="menu"
            aria-expanded={openMenu === bucket.id}
          >
            {bucket.title}
          </button>
          {openMenu === bucket.id && (
            <div
              className="app-menu-bar__dropdown"
              role="menu"
              aria-label={`${bucket.title} menu`}
            >
              {(menuById.get(bucket.id)?.rows ?? []).map((row, rowIndex) => {
                if (row.kind === 'separator') {
                  return (
                    <div
                      key={`${bucket.id}__sep_${rowIndex}`}
                      role="separator"
                      className="app-menu-bar__separator"
                    />
                  )
                }
                const clickable = !!(row.onSelect && !row.disabled && !row.inert)
                if (row.inert) {
                  return (
                    <div key={row.id} className="app-menu-bar__item app-menu-bar__item--inert" role="presentation">
                      <span>{row.label}</span>
                    </div>
                  )
                }
                return (
                  <button
                    key={row.id}
                    type="button"
                    className={`app-menu-bar__item${row.disabled ? ' app-menu-bar__item--disabled' : ''}`}
                    role="menuitem"
                    disabled={row.disabled ?? !clickable}
                    onClick={
                      clickable
                        ? () => {
                            setOpenMenu(null)
                            row.onSelect?.()
                          }
                        : undefined
                    }
                  >
                    <span>{row.label}</span>
                    {row.shortcut ? <kbd>{row.shortcut}</kbd> : null}
                  </button>
                )
              })}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}
