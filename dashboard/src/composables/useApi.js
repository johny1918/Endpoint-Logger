import { ref } from 'vue'

const API_BASE = '/api'

/**
 * Build query string from filter parameters
 * @param {Object} params - Filter parameters
 * @returns {string} Query string (without leading ?)
 */
function buildQueryString(params) {
  const searchParams = new URLSearchParams()

  if (params.limit !== undefined) {
    searchParams.append('limit', params.limit)
  }
  if (params.offset !== undefined) {
    searchParams.append('offset', params.offset)
  }
  if (params.from_timestamp) {
    searchParams.append('from_timestamp', params.from_timestamp)
  }
  if (params.to_timestamp) {
    searchParams.append('to_timestamp', params.to_timestamp)
  }
  if (params.methods && params.methods.length > 0) {
    params.methods.forEach(m => searchParams.append('methods', m))
  }
  if (params.path_pattern) {
    searchParams.append('path_pattern', params.path_pattern)
  }
  if (params.status_codes && params.status_codes.length > 0) {
    params.status_codes.forEach(s => searchParams.append('status_codes', s))
  }

  return searchParams.toString()
}

/**
 * API client composable for interacting with the backend REST API
 */
export function useApi() {
  const loading = ref(false)
  const error = ref(null)
  let abortController = null

  /**
   * Cancel any in-flight request
   */
  function cancelRequest() {
    if (abortController) {
      abortController.abort()
      abortController = null
    }
  }

  /**
   * Fetch recent logs with optional limit
   * @param {number} limit - Maximum number of logs to fetch
   * @returns {Promise<Array>} Array of log entries
   */
  async function fetchRecentLogs(limit = 50) {
    return fetchLogs({ limit })
  }

  /**
   * Fetch logs with full filter support
   * @param {Object} options - Filter options
   * @param {number} options.limit - Max results (default: 50)
   * @param {number} options.offset - Skip N results (default: 0)
   * @param {string} options.from_timestamp - ISO timestamp for start range
   * @param {string} options.to_timestamp - ISO timestamp for end range
   * @param {string[]} options.methods - Filter by HTTP methods
   * @param {string} options.path_pattern - Filter by path pattern (LIKE)
   * @param {number[]} options.status_codes - Filter by status codes
   * @returns {Promise<Array>} Array of log entries
   */
  async function fetchLogs(options = {}) {
    cancelRequest()
    abortController = new AbortController()

    loading.value = true
    error.value = null

    const params = {
      limit: options.limit ?? 50,
      offset: options.offset ?? 0,
      ...options
    }

    try {
      const queryString = buildQueryString(params)
      const url = queryString ? `${API_BASE}/logs?${queryString}` : `${API_BASE}/logs`

      const response = await fetch(url, {
        signal: abortController.signal
      })

      if (!response.ok) {
        throw new Error(`HTTP error: ${response.status}`)
      }

      const data = await response.json()
      return data
    } catch (e) {
      if (e.name === 'AbortError') {
        console.log('[API] Request cancelled')
        return []
      }
      console.error('[API] Failed to fetch logs:', e)
      error.value = e.message
      return []
    } finally {
      loading.value = false
      abortController = null
    }
  }

  /**
   * Get count of logs matching filters
   * @param {Object} options - Same filter options as fetchLogs (without limit/offset)
   * @returns {Promise<number>} Count of matching logs
   */
  async function fetchLogCount(options = {}) {
    loading.value = true
    error.value = null

    try {
      const queryString = buildQueryString(options)
      const url = queryString ? `${API_BASE}/logs/count?${queryString}` : `${API_BASE}/logs/count`

      const response = await fetch(url)

      if (!response.ok) {
        throw new Error(`HTTP error: ${response.status}`)
      }

      const data = await response.json()
      return data.count ?? 0
    } catch (e) {
      console.error('[API] Failed to fetch log count:', e)
      error.value = e.message
      return 0
    } finally {
      loading.value = false
    }
  }

  /**
   * Fetch a single log by request ID
   * @param {string} requestId - The request_id to fetch
   * @returns {Promise<Object|null>} Log entry or null if not found
   */
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

  /**
   * Clear the current error state
   */
  function clearError() {
    error.value = null
  }

  return {
    // State
    loading,
    error,
    // Methods
    fetchRecentLogs,
    fetchLogs,
    fetchLogCount,
    fetchLogById,
    cancelRequest,
    clearError
  }
}
