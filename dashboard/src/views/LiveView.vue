<script setup>
import { ref, computed } from 'vue'
import { useWebSocket } from '../composables/useWebSocket'
import { useApi } from '../composables/useApi'
import LogStream from '../components/LogStream.vue'
import LogDetail from '../components/LogDetail.vue'
import Filter from '../components/Filter.vue'

const { logs, connected, clearLogs } = useWebSocket()
const { fetchRecentLogs, loading } = useApi()

const selectedLog = ref(null)
const showDetail = ref(false)
const filters = ref({
  methods: [],
  statuses: [],
  pathPattern: ''
})

// Load historical logs on mount
fetchRecentLogs(50).then(historicalLogs => {
  const existingIds = new Set(logs.value.map(l => l.request_id))
  const newLogs = historicalLogs.filter(l => !existingIds.has(l.request_id))
  logs.value.push(...newLogs)
  logs.value.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp))
})

const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    // Filter by methods
    if (filters.value.methods.length > 0 && !filters.value.methods.includes(log.method)) {
      return false
    }

    // Filter by status codes
    if (filters.value.statuses.length > 0 && !filters.value.statuses.includes(log.status_code)) {
      return false
    }

    // Filter by path pattern
    if (filters.value.pathPattern.trim()) {
      const pattern = filters.value.pathPattern.toLowerCase()
      const path = log.path.toLowerCase()
      if (!path.includes(pattern)) {
        return false
      }
    }

    return true
  })
})

const selectLog = (log) => {
  selectedLog.value = log
  showDetail.value = true
}

const closeDetail = () => {
  showDetail.value = false
  selectedLog.value = null
}

const handleClear = () => {
  clearLogs()
}

const updateFilters = (newFilters) => {
  filters.value = newFilters
}
</script>

<template>
  <div class="live-view">
    <Filter
      :logs="logs"
      @update:filters="updateFilters"
    />

    <LogStream
      :logs="filteredLogs"
      :loading="loading"
      :show-clear="true"
      title="Live Requests"
      @select="selectLog"
      @clear="handleClear"
    />

    <LogDetail
      v-if="showDetail"
      :log="selectedLog"
      @close="closeDetail"
    />
  </div>
</template>

<style scoped>
.live-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
