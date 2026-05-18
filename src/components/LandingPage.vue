<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  selectFolder,
  scanMdFiles,
  saveLastDir,
  loadLastDir,
} from '../composables/useFileSystem'
import type { MdFileEntry } from '../types'

const emit = defineEmits<{
  'folder-opened': [dir: string, files: MdFileEntry[]]
}>()

const lastDir = ref<string | null>(null)
const isLoading = ref(false)
const errorMsg = ref('')

const lastDirName = computed(() => {
  if (!lastDir.value) return ''
  const parts = lastDir.value.replace(/\\/g, '/').split('/')
  return parts[parts.length - 1] || parts[parts.length - 2] || lastDir.value
})

onMounted(() => {
  lastDir.value = loadLastDir()
})

async function handleOpen() {
  errorMsg.value = ''
  const dir = await selectFolder()
  if (!dir) return
  await openDirectory(dir)
}

async function handleOpenLast() {
  if (!lastDir.value) return
  errorMsg.value = ''
  await openDirectory(lastDir.value)
}

async function openDirectory(dir: string) {
  isLoading.value = true
  try {
    const mdFiles = await scanMdFiles(dir)
    if (mdFiles.length === 0) {
      errorMsg.value = 'No .md files found in this directory.'
      return
    }
    saveLastDir(dir)
    emit('folder-opened', dir, mdFiles)
  } catch (err) {
    errorMsg.value = `Failed to scan: ${err}`
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="landing">
    <div class="landing-card">
      <div class="landing-icon">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
      </div>

      <h1 class="landing-title">MD Viewer</h1>
      <p class="landing-subtitle">Lightweight Markdown file browser</p>

      <button class="btn-primary" :disabled="isLoading" @click="handleOpen">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        {{ isLoading ? 'Scanning...' : 'Open Folder' }}
      </button>

      <button
        v-if="lastDir && !isLoading"
        class="btn-ghost"
        @click="handleOpenLast"
      >
        Reopen: {{ lastDirName }}
      </button>

      <p v-if="errorMsg" class="error-msg">{{ errorMsg }}</p>
    </div>
  </div>
</template>

<style scoped>
.landing {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background: var(--bg-raised);
}

.landing-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--gap-3);
  padding: 48px 56px;
  background: var(--bg);
  border: 0.5px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow-card);
  text-align: center;
  max-width: 380px;
}

.landing-icon {
  color: var(--fg-subtle);
  margin-bottom: var(--gap-1);
}

.landing-title {
  font-family: var(--font-label);
  font-size: 26px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--fg);
}

.landing-subtitle {
  font-family: var(--font-label);
  font-size: var(--text-sm);
  text-transform: uppercase;
  letter-spacing: var(--letter-uppercase);
  color: var(--fg-muted);
  margin-bottom: var(--gap-3);
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: var(--gap-2);
  padding: var(--gap-2) var(--gap-6);
  background: var(--fg);
  color: var(--fg-on-dark);
  border: none;
  border-radius: var(--radius);
  font-family: var(--font-sans);
  font-size: var(--text-md);
  font-weight: 500;
  cursor: pointer;
  transition: background-color var(--t-fast);
}
.btn-primary:hover:not(:disabled) {
  background: #1a1a1a;
}
.btn-primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-ghost {
  padding: 6px var(--gap-4);
  background: none;
  border: 0.5px solid var(--border);
  border-radius: var(--radius);
  font-family: var(--font-sans);
  font-size: var(--text-base);
  color: var(--fg-muted);
  cursor: pointer;
  transition: background-color var(--t-fast), border-color var(--t-fast), color var(--t-fast);
  max-width: 260px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.btn-ghost:hover {
  background: var(--surface-hover);
  border-color: var(--border-strong);
  color: var(--fg);
}

.error-msg {
  font-size: var(--text-base);
  color: #E53935;
  margin-top: var(--gap-1);
}
</style>
