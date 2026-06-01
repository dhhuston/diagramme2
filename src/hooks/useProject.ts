import { useCallback, useEffect, useState } from 'react'

import { getProject, type ProjectState } from '../tauriIpc'

export function useProject() {
  const [project, setProject] = useState<ProjectState | null>(null)
  const [error, setError] = useState<string | null>(null)

  const refreshProject = useCallback(async () => {
    try {
      const next = await getProject()
      setProject(next)
      setError(null)
      return next
    } catch (err) {
      setError(String(err))
      throw err
    }
  }, [])

  useEffect(() => {
    void refreshProject()
  }, [refreshProject])

  return {
    project,
    setProject,
    refreshProject,
    error,
  }
}
