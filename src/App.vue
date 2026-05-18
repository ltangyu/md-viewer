<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { MdFileEntry, ViewState } from './types'
import { scanMdFiles, saveLastDir } from './composables/useFileSystem'
import TitleBar from './components/TitleBar.vue'
import LandingPage from './components/LandingPage.vue'
import FileBrowser from './components/FileBrowser.vue'

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
  <div class="app-frame">
    <TitleBar />
    <div class="app-body">
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
</template>

<style scoped>
.app-frame {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg);
}

.app-body {
  flex: 1;
  overflow: hidden;
}
</style>
