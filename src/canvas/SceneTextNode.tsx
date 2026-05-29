import { useCallback } from 'react'
import { Text } from 'react-konva'
import type Konva from 'konva'

import { applySceneTextAnchor, sceneCapHeightToFontSizePx } from './sceneRenderUtils'
import type { SceneText } from './sceneTypes'

type SceneTextNodeProps = {
  text: SceneText
}

function scheduleTextAnchor(node: Konva.Text, halign: SceneText['halign'], valign: SceneText['valign']) {
  const apply = () => applySceneTextAnchor(node, halign, valign)
  apply()
  requestAnimationFrame(apply)
  void document.fonts.ready.then(apply)
}

/** SceneText position is an insertion anchor (halign/valign), not Konva's top-left. */
export function SceneTextNode({ text }: SceneTextNodeProps) {
  const ref = useCallback(
    (node: Konva.Text | null) => {
      if (node) {
        scheduleTextAnchor(node, text.halign, text.valign)
      }
    },
    [text.content, text.font, text.halign, text.height_px, text.valign],
  )

  const fontSize = sceneCapHeightToFontSizePx(text.height_px)

  return (
    <Text
      ref={ref}
      x={text.position.x}
      y={text.position.y}
      text={text.content}
      fontFamily={`"${text.font}", "Arial Narrow", Arial, sans-serif`}
      fontSize={fontSize}
      lineHeight={1}
      align="left"
      verticalAlign="top"
      fill="#000000"
      listening={false}
      perfectDrawEnabled={false}
    />
  )
}
