<script setup>
import { computed } from 'vue'
import { RouterLink, RouterView } from 'vue-router'
import { useWebSocket } from './composables/useWebSocket'

const { connected, connectionStatus } = useWebSocket()

const statusColor = computed(() => {
  if (connected.value) return 'var(--success)'
  return 'var(--error)'
})
</script>

<template>
  <header class="header">
    <div class="header-left">
      <h1>Endpoint <span class="accent">Logger</span></h1>
      <nav class="nav">
        <RouterLink to="/" class="nav-link">Live</RouterLink>
        <RouterLink to="/history" class="nav-link">History</RouterLink>
      </nav>
    </div>
    <div class="header-right">
      <div class="connection-status">
        <span class="status-dot" :style="{ background: statusColor }"></span>
        <span class="status-text">{{ connectionStatus }}</span>
      </div>
    </div>
  </header>

  <main class="main">
    <RouterView />
  </main>
</template>

<style scoped>
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 32px;
}

.header h1 {
  font-size: 1.5rem;
  font-weight: 600;
}

.header .accent {
  color: var(--accent);
}

.nav {
  display: flex;
  gap: 8px;
}

.nav-link {
  color: var(--text-secondary);
  text-decoration: none;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.875rem;
  transition: all 0.15s;
}

.nav-link:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.nav-link.router-link-active {
  background: var(--accent);
  color: white;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.main {
  flex: 1;
  padding: 24px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
