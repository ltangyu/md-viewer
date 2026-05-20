<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { MdFileEntry, ViewState } from './types'
import { scanMdFiles, saveLastDir } from './composables/useFileSystem'
import TitleBar from './components/TitleBar.vue'
import LandingPage from './components/LandingPage.vue'
import FileBrowser from './components/FileBrowser.vue'

const win = getCurrentWindow()

/** Drag the window from the transparent outer padding ring. */
async function onPaddingMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  if (e.target !== e.currentTarget) return
  await win.startDragging()
}

const view = ref<ViewState>('landing')
const files = ref<MdFileEntry[]>([])
const currentDir = ref('')
const initialFilePath = ref<string | null>(null)

function handleFolderOpened(dir: string, mdFiles: MdFileEntry[]) {
  currentDir.value = dir
  files.value = mdFiles
  view.value = 'browser'
}

function handleChangeFolder() {
  initialFilePath.value = null
  view.value = 'landing'
}

/**
 * Open a .md file directly: scan its parent directory,
 * switch to browser view, and navigate to the target file.
 */
async function openFileDirectly(filePath: string) {
  const normalized = filePath.replace(/\\/g, '/')
  const dir = normalized.substring(0, normalized.lastIndexOf('/'))
  if (!dir) return

  try {
    const mdFiles = await scanMdFiles(dir)
    if (mdFiles.length === 0) return

    saveLastDir(dir)
    currentDir.value = dir
    files.value = mdFiles
    initialFilePath.value = filePath
    view.value = 'browser'
  } catch {
    // silently ignore — user can still open manually
  }
}

onMounted(async () => {
  // 1) Check if launched with a .md file argument (double-click / Open With)
  try {
    const launchFile = await invoke<string | null>('get_launch_file')
    if (launchFile) {
      await openFileDirectly(launchFile)
    }
  } catch {
    // command not available or failed
  }

  // 2) Listen for files opened while the app is already running
  //    (single-instance plugin forwards the path via event)
  await listen<string>('open-file', async (event) => {
    await openFileDirectly(event.payload)
  })
})
</script>

<template>
  <div class="shell-padding" @mousedown="onPaddingMouseDown" data-tauri-drag-region>
    <div class="shell">
      <TitleBar />
      <div class="shell-body">
        <LandingPage
          v-if="view === 'landing'"
          @folder-opened="handleFolderOpened"
        />
        <FileBrowser
          v-else
          :files="files"
          :current-dir="currentDir"
          :initial-file-path="initialFilePath"
          @change-folder="handleChangeFolder"
          @folder-opened="handleFolderOpened"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Outer ring — fully transparent so the inner shell's box-shadow can fade out. */
.shell-padding {
  height: 100%;
  padding: var(--shell-padding);
  background: transparent;
  overflow: hidden;
}

/* The visible card — rounded corners + layered drop shadow. */
/* PDF §1 SURFACES: shell uses #fafafa (--bg-raised); inner content blocks use #ffffff. */
.shell {
  height: 100%;
  background: var(--bg-raised);
  border-radius: var(--window-radius);
  box-shadow: var(--shadow-shell);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.shell-body {
  flex: 1;
  overflow: hidden;
}
</style>
