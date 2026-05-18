<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()
const isMaximized = ref(false)

async function syncMaximized() {
  isMaximized.value = await appWindow.isMaximized()
}

async function handleMinimize() {
  await appWindow.minimize()
}

async function handleToggleMaximize() {
  await appWindow.toggleMaximize()
  await syncMaximized()
}

async function handleClose() {
  await appWindow.close()
}

/* Double-click title bar to toggle maximize */
async function handleDblClick() {
  await handleToggleMaximize()
}

let unlisten: (() => void) | null = null

onMounted(async () => {
  await syncMaximized()
  unlisten = await appWindow.onResized(async () => {
    await syncMaximized()
  })
})

onUnmounted(() => {
  unlisten?.()
})
</script>

<template>
  <header class="titlebar" data-tauri-drag-region @dblclick="handleDblClick">
    <div class="titlebar-title" data-tauri-drag-region>MD Viewer</div>

    <div class="titlebar-controls">
      <button class="titlebar-btn" title="Minimize" @click="handleMinimize">
        <svg width="10" height="10" viewBox="0 0 10 10">
          <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>

      <button class="titlebar-btn" title="Maximize" @click="handleToggleMaximize">
        <svg v-if="!isMaximized" width="10" height="10" viewBox="0 0 10 10">
          <rect x="1" y="1" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
        </svg>
        <svg v-else width="10" height="10" viewBox="0 0 10 10">
          <rect x="2.5" y="0.5" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/>
          <rect x="0.5" y="2.5" width="7" height="7" rx="1" fill="var(--bg)" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>

      <button class="titlebar-btn titlebar-btn--close" title="Close" @click="handleClose">
        <svg width="10" height="10" viewBox="0 0 10 10">
          <line x1="1.5" y1="1.5" x2="8.5" y2="8.5" stroke="currentColor" stroke-width="1.2"/>
          <line x1="8.5" y1="1.5" x2="1.5" y2="8.5" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--titlebar-h);
  padding-left: var(--gap-4);
  background: var(--bg);
  border-bottom: 0.5px solid var(--border-soft);
  flex-shrink: 0;
  user-select: none;
  -webkit-user-select: none;
  /* Top corners match the window radius */
  border-radius: var(--window-radius) var(--window-radius) 0 0;
}

.titlebar-title {
  font-family: var(--font-label);
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--fg-muted);
  letter-spacing: 0.02em;
  pointer-events: none;
}

.titlebar-controls {
  display: flex;
  align-items: stretch;
  height: 100%;
}

.titlebar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--fg-muted);
  cursor: pointer;
  transition: background-color var(--t-fast), color var(--t-fast);
}
.titlebar-btn:hover {
  background: var(--surface-hover);
  color: var(--fg);
}

.titlebar-btn--close:hover {
  background: #e53935;
  color: #ffffff;
}
</style>
