import { ref } from 'vue'

const API_BASE = '/api'

export function useApi() {
  const loading = ref(false)
  const error = ref(null)

  async function fetchRecentLogs(limit = 50) {
    loading.value = true
    error.value = null

    try {
      const response = await fetch(`${API_BASE}/logs?limit=${limit}`)
      if (!response.ok) {
        throw new Error(`HTTP error: ${response.status}`)
      }
      const data = await response.json()
      return data
    } catch (e) {
      console.error('[API] Failed to fetch logs:', e)
      error.value = e.message
      return []
    } finally {
      loading.value = false
    }
  }

  async function fetchLogById(requestId) {
    loading.value = true
    error.value = null

    try {
      const response = await fetch(`${API_BASE}/logs/${requestId}`)
      if (!response.ok) {
        if (response.status === 404) {
          return null
        }
        throw new Error(`HTTP error: ${response.status}`)
      }
      const data = await response.json()
      return data
    } catch (e) {
      console.error('[API] Failed to fetch log:', e)
      error.value = e.message
      return null
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    error,
    fetchRecentLogs,
    fetchLogById
  }
}
