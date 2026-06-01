import React, { useState, useRef, useEffect } from 'react'
import './TabStrip.css'

export interface Tab {
  id: string
  name: string
}

interface TabStripProps {
  tabs: Tab[]
  activeId: string
  onSwitch: (id: string) => void
  onAdd: () => void
  onRemove: (id: string) => void
  onRename: (id: string, newName: string) => void
}

export const TabStrip: React.FC<TabStripProps> = ({
  tabs,
  activeId,
  onSwitch,
  onAdd,
  onRemove,
  onRename,
}) => {
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editValue, setEditValue] = useState('')
  const editInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (editingId && editInputRef.current) {
      editInputRef.current.focus()
      editInputRef.current.select()
    }
  }, [editingId])

  const startEditing = (tab: Tab) => {
    setEditingId(tab.id)
    setEditValue(tab.name)
  }

  const finishEditing = () => {
    if (editingId && editValue.trim()) {
      onRename(editingId, editValue.trim())
    }
    setEditingId(null)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') finishEditing()
    if (e.key === 'Escape') setEditingId(null)
  }

  return (
    <div className="tab-strip">
      <div className="tabs-container" role="tablist" aria-label="Project sheets">
        {tabs.map((tab) => {
          const isActive = tab.id === activeId
          return (
            <div
              key={tab.id}
              className={`tab-item${isActive ? ' tab-item--active' : ''}`}
            >
              {editingId === tab.id ? (
                <input
                  ref={editInputRef}
                  className="tab-edit-input"
                  value={editValue}
                  onChange={(e) => setEditValue(e.target.value)}
                  onBlur={finishEditing}
                  onKeyDown={handleKeyDown}
                  aria-label="Rename sheet"
                />
              ) : (
                <button
                  type="button"
                  role="tab"
                  className="tab-btn"
                  aria-selected={isActive}
                  aria-controls={isActive ? 'rf-canvas' : undefined}
                  onClick={() => onSwitch(tab.id)}
                  onDoubleClick={() => startEditing(tab)}
                >
                  <span className="tab-name">{tab.name}</span>
                </button>
              )}
              {tabs.length > 1 && (
                <button
                  type="button"
                  className="tab-close"
                  aria-label={`Delete sheet ${tab.name}`}
                  onClick={(e) => {
                    e.stopPropagation()
                    if (window.confirm(`Delete sheet "${tab.name}"?`)) {
                      onRemove(tab.id)
                    }
                  }}
                >
                  <svg viewBox="0 0 16 16" width="10" height="10" aria-hidden="true">
                    <path
                      d="M3 3L13 13M3 13L13 3"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                    />
                  </svg>
                </button>
              )}
            </div>
          )
        })}
        <button className="tab-add" onClick={onAdd} title="Add Sheet" aria-label="Add Sheet">
          <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
            <path
              d="M8 3V13M3 8H13"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
        </button>
      </div>
    </div>
  )
}
