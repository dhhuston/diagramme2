/** Dev-only fixture paths served by Vite `/fixtures/` middleware. */
export const DEV_FIXTURES = {
  compGym: '/fixtures/golden/Comp Gym F102A.diagramme',
  cafeteria: '/fixtures/diagrams/cafeteria-d104a.diagramme',
  splitFaceDemo: '/fixtures/diagrams/split-face-demo.diagramme',
} as const

export async function fetchDevFixture(path: string): Promise<string> {
  const res = await fetch(path)
  if (!res.ok) throw new Error(`fixture fetch ${res.status}: ${path}`)
  return res.text()
}
