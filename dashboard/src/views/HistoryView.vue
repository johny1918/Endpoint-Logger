<script setup>
import { ref, onMounted } from 'vue'
import { useApi } from '../composables/useApi'
import LogStream from '../components/LogStream.vue'
import LogDetail from '../components/LogDetail.vue'

const { fetchRecentLogs, loading } = useApi()

const logs = ref([])
const selectedLog = ref(null)
const showDetail = ref(false)

onMounted(async () => {
  logs.value = await fetchRecentLogs(100)
})

const selectLog = (log) => {
  selectedLog.value = log
  showDetail.value = true
}

const closeDetail = () => {
  showDetail.value = false
  selectedLog.value = null
}

const refresh = async () => {
  logs.value = await fetchRecentLogs(100)
}
</script>

<template>
  <div class="history-view">
    <div class="history-header">
      <h2>Request History</h2>
      <button class="refresh-btn" @click="refresh" :disabled="loading">
        {{ loading ? 'Loading...' : 'Refresh' }}
      </button>
    </div>

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
.history-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.history-header h2 {
  font-size: 1.25rem;
  font-weight: 600;
}

.refresh-btn {
  background: var(--accent);
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.875rem;
}

.refresh-btn:hover {
  opacity: 0.9;
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
