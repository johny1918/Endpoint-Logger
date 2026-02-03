<script setup>
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { RouterLink, RouterView, useRouter } from 'vue-router'
import { useWebSocket } from './composables/useWebSocket'

const router = useRouter()
const { connected, connectionStatus } = useWebSocket()

const statusColor = computed(() => {
  if (connected.value) return 'var(--success)'
  return 'var(--error)'
})

/**
 * Keyboard shortcuts
 * l - Go to Live view
 * h - Go to History view
 */
function handleKeyboardShortcuts(e) {
  // Only trigger if not typing in an input
  if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return
  
  if (e.key === 'l' || e.key === 'L') {
    router.push('/')
  } else if (e.key === 'h' || e.key === 'H') {
    router.push('/history')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeyboardShortcuts)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyboardShortcuts)
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
  box-shadow: 0 1px 3px var(--shadow);
  animation: slideInDown var(--transition-base);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 32px;
}

.header h1 {
  font-size: 1.5rem;
  font-weight: 700;
  letter-spacing: -0.5px;
  background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
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
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 0.875rem;
  font-weight: 500;
  transition: all var(--transition-fast);
  position: relative;
  cursor: pointer;
}

.nav-link:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  transform: translateY(-1px);
}

.nav-link.router-link-active {
  background: var(--accent);
  color: white;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
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
  padding: 6px 12px;
  background: var(--bg-tertiary);
  border-radius: 6px;
  transition: all var(--transition-fast);
}

.connection-status:hover {
  background: var(--bg-hover);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  box-shadow: 0 0 8px currentColor;
}

.main {
  flex: 1;
  padding: 24px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  animation: fadeIn var(--transition-base);
}

/* Responsive Design */
@media (max-width: 768px) {
  .header {
    flex-direction: column;
    gap: 16px;
    padding: 12px 16px;
  }

  .header-left {
    width: 100%;
    gap: 16px;
  }

  .header h1 {
    font-size: 1.25rem;
  }

  .nav {
    gap: 4px;
  }

  .nav-link {
    padding: 6px 12px;
    font-size: 0.8125rem;
  }

  .header-right {
    width: 100%;
  }

  .connection-status {
    flex: 1;
    justify-content: center;
  }

  .main {
    padding: 16px;
  }
}

@media (max-width: 480px) {
  .header {
    padding: 12px 12px;
  }

  .header-left {
    gap: 12px;
  }

  .header h1 {
    font-size: 1.1rem;
  }

  .nav-link {
    padding: 4px 10px;
    font-size: 0.75rem;
  }

  .main {
    padding: 12px;
  }
}
</style>
