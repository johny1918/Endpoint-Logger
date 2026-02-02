<script setup>
import { ref } from 'vue'
import { useWebSocket } from '../composables/useWebSocket'
import { useApi } from '../composables/useApi'
import LogStream from '../components/LogStream.vue'
import LogDetail from '../components/LogDetail.vue'

const { logs, connected } = useWebSocket()
const { fetchRecentLogs, loading } = useApi()

const selectedLog = ref(null)
const showDetail = ref(false)

// Load historical logs on mount
fetchRecentLogs(50).then(historicalLogs => {
  const existingIds = new Set(logs.value.map(l => l.request_id))
  const newLogs = historicalLogs.filter(l => !existingIds.has(l.request_id))
  logs.value.push(...newLogs)
  logs.value.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp))
})

const selectLog = (log) => {
  selectedLog.value = log
  showDetail.value = true
}

const closeDetail = () => {
  showDetail.value = false
  selectedLog.value = null
}
</script>

<template>
  <div class="live-view">
    <LogStream
      :logs="logs"
      :loading="loading"
      @select="selectLog"
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
