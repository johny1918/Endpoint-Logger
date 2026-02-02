<script setup>
import { computed, ref } from 'vue'

const props = defineProps({
  log: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['close'])

const activeTab = ref('request')
const copySuccess = ref(null)

function formatTimestamp(timestamp) {
  return new Date(timestamp).toLocaleString()
}

function formatJson(data) {
  if (!data) return null
  try {
    const parsed = JSON.parse(data)
    return JSON.stringify(parsed, null, 2)
  } catch {
    return data
  }
}

function isJson(data) {
  if (!data) return false
  try {
    JSON.parse(data)
    return true
  } catch {
    return false
  }
}

const requestBody = computed(() => formatJson(props.log.request_body))
const responseBody = computed(() => formatJson(props.log.response_body))

function getStatusClass(status) {
  if (status >= 200 && status < 300) return 'status-success'
  if (status >= 300 && status < 400) return 'status-redirect'
  if (status >= 400 && status < 500) return 'status-client-error'
  if (status >= 500) return 'status-server-error'
  return ''
}

async function copyToClipboard(text, type) {
  try {
    await navigator.clipboard.writeText(text)
    copySuccess.value = type
    setTimeout(() => {
      copySuccess.value = null
    }, 2000)
  } catch (e) {
    console.error('Failed to copy:', e)
  }
}

function handleBackdropClick(e) {
  if (e.target === e.currentTarget) {
    emit('close')
  }
}

function handleKeydown(e) {
  if (e.key === 'Escape') {
    emit('close')
  }
}
</script>

<template>
  <div class="modal-backdrop" @click="handleBackdropClick" @keydown="handleKeydown" tabindex="0">
    <div class="modal">
      <div class="modal-header">
        <div class="header-info">
          <span class="method" :class="'method-' + log.method.toLowerCase()">{{ log.method }}</span>
          <span class="path mono">{{ log.path }}</span>
          <span class="status" :class="getStatusClass(log.status_code)">{{ log.status_code }}</span>
        </div>
        <button class="close-btn" @click="emit('close')">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="modal-meta">
        <div class="meta-item">
          <span class="meta-label">Request ID</span>
          <span class="meta-value mono">{{ log.request_id }}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Timestamp</span>
          <span class="meta-value">{{ formatTimestamp(log.timestamp) }}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Duration</span>
          <span class="meta-value">{{ log.duration_ms }}ms</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Client IP</span>
          <span class="meta-value mono">{{ log.client_ip }}</span>
        </div>
      </div>

      <div class="tabs">
        <button
          class="tab"
          :class="{ active: activeTab === 'request' }"
          @click="activeTab = 'request'"
        >
          Request
        </button>
        <button
          class="tab"
          :class="{ active: activeTab === 'response' }"
          @click="activeTab = 'response'"
        >
          Response
        </button>
      </div>

      <div class="modal-body">
        <!-- Request Tab -->
        <div v-if="activeTab === 'request'" class="tab-content">
          <div class="section">
            <div class="section-header">
              <h3>Headers</h3>
              <button
                class="copy-btn"
                @click="copyToClipboard(JSON.stringify(log.request_headers, null, 2), 'req-headers')"
              >
                {{ copySuccess === 'req-headers' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <div class="headers">
              <div v-for="(value, key) in log.request_headers" :key="key" class="header-row">
                <span class="header-key">{{ key }}</span>
                <span class="header-value mono">{{ value }}</span>
              </div>
              <div v-if="Object.keys(log.request_headers || {}).length === 0" class="empty-section">
                No headers
              </div>
            </div>
          </div>

          <div v-if="log.query_string" class="section">
            <h3>Query String</h3>
            <pre class="code-block mono">{{ log.query_string }}</pre>
          </div>

          <div class="section">
            <div class="section-header">
              <h3>Body</h3>
              <button
                v-if="log.request_body"
                class="copy-btn"
                @click="copyToClipboard(requestBody, 'req-body')"
              >
                {{ copySuccess === 'req-body' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <pre v-if="requestBody" class="code-block mono" :class="{ json: isJson(log.request_body) }">{{ requestBody }}</pre>
            <div v-else class="empty-section">No body</div>
          </div>
        </div>

        <!-- Response Tab -->
        <div v-if="activeTab === 'response'" class="tab-content">
          <div class="section">
            <div class="section-header">
              <h3>Headers</h3>
              <button
                class="copy-btn"
                @click="copyToClipboard(JSON.stringify(log.response_headers, null, 2), 'res-headers')"
              >
                {{ copySuccess === 'res-headers' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <div class="headers">
              <div v-for="(value, key) in log.response_headers" :key="key" class="header-row">
                <span class="header-key">{{ key }}</span>
                <span class="header-value mono">{{ value }}</span>
              </div>
              <div v-if="Object.keys(log.response_headers || {}).length === 0" class="empty-section">
                No headers
              </div>
            </div>
          </div>

          <div class="section">
            <div class="section-header">
              <h3>Body</h3>
              <button
                v-if="log.response_body"
                class="copy-btn"
                @click="copyToClipboard(responseBody, 'res-body')"
              >
                {{ copySuccess === 'res-body' ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <pre v-if="responseBody" class="code-block mono" :class="{ json: isJson(log.response_body) }">{{ responseBody }}</pre>
            <div v-else class="empty-section">No body</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  z-index: 100;
}

.modal {
  background: var(--bg-secondary);
  border-radius: 16px;
  width: 100%;
  max-width: 800px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border);
}

.header-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.method {
  font-weight: 700;
  font-size: 0.875rem;
}

.method-get { color: var(--success); }
.method-post { color: var(--accent); }
.method-put { color: var(--warning); }
.method-patch { color: #a855f7; }
.method-delete { color: var(--error); }

.path {
  font-size: 0.875rem;
  color: var(--text-primary);
}

.status {
  font-weight: 600;
  font-size: 0.875rem;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--bg-tertiary);
}

.status-success { color: var(--success); }
.status-redirect { color: var(--accent); }
.status-client-error { color: var(--warning); }
.status-server-error { color: var(--error); }

.close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.modal-meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
  padding: 16px 24px;
  background: var(--bg-primary);
  border-bottom: 1px solid var(--border);
}

.meta-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.meta-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.meta-value {
  font-size: 0.875rem;
  color: var(--text-primary);
}

.tabs {
  display: flex;
  padding: 0 24px;
  border-bottom: 1px solid var(--border);
}

.tab {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.875rem;
  font-weight: 500;
  padding: 12px 16px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
}

.tab:hover {
  color: var(--text-primary);
}

.tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section h3 {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.copy-btn {
  background: var(--bg-tertiary);
  border: none;
  color: var(--text-muted);
  font-size: 0.75rem;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
}

.copy-btn:hover {
  color: var(--text-primary);
}

.headers {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.header-row {
  display: flex;
  gap: 12px;
  font-size: 0.8125rem;
}

.header-key {
  color: var(--accent);
  flex-shrink: 0;
}

.header-value {
  color: var(--text-primary);
  word-break: break-all;
}

.code-block {
  background: var(--bg-primary);
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  font-size: 0.8125rem;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
}

.code-block.json {
  color: var(--success);
}

.empty-section {
  color: var(--text-muted);
  font-size: 0.875rem;
  font-style: italic;
}
</style>
