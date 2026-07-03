<script setup lang="ts">
import { ref, watch, computed, onBeforeUnmount } from 'vue'
import { MdEditor } from 'md-editor-v3'
import 'md-editor-v3/lib/style.css'
import { useTreeStore } from '../stores/treeStore'
import { useConfigStore } from '../stores/configStore'
import { useAppStatusStore } from '../stores/appStatusStore'
import { useAutoSave } from '../composables/useAutoSave'
import LaySplitPanel from '../components/LaySplitPanel.vue'
import NotebookTree from '../components/NotebookTree.vue'
import { useI18n } from 'vue-i18n'
import { join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { buildImagePlaceholder, sanitizeHtml as sanitizeHtmlExt } from '@/services/editor/editor-extension'

const tree = useTreeStore()
const cfg = useConfigStore()
const appStatus = useAppStatusStore()
const { t } = useI18n()
const saveMsg = ref('')
let saveMsgTimer: number | null = null
function showSaved() {
  saveMsg.value = t('editor.saved')
  if (saveMsgTimer) clearTimeout(saveMsgTimer)
  saveMsgTimer = window.setTimeout(() => { saveMsg.value = '' }, 2000)
}

// ── Editor content (local mirror) ────────────────────────────
const content = ref('')
const blockNextChange = ref(false)

type SavePayload = { nodeId: string; content: string }
type MdEditorFooter = '=' | 'markdownTotal' | 'scrollSwitch' | number

// ── Auto-save (3s debounce) ───────────────────────────────────
const { trigger, flush } = useAutoSave<SavePayload>(async (payload) => {
  appStatus.setSaving()
  try {
    await tree.saveNodeContent(payload.nodeId, payload.content)
    appStatus.setSaved()
    showSaved()
  } catch (error) {
    appStatus.setError()
    throw error
  }
}, 3000)

watch(() => tree.current, async (node) => {
  await flush()
  if ((node?.id ?? null) !== tree.currentId) return
  if (!node) { content.value = ''; return }
  blockNextChange.value = true
  content.value = node.content ?? ''
})

watch(() => appStatus.saveRequestId, async (requestId) => {
  if (!requestId) return
  await onSave(content.value)
})

function onChange(v: string) {
  if (blockNextChange.value) { blockNextChange.value = false; return }
  if (tree.current && v === (tree.current.content ?? '')) return
  content.value = v
  if (tree.currentId) {
    appStatus.setSaving()
    trigger({ nodeId: tree.currentId, content: v })
  }
}

async function onSave(v: string) {
  const nodeId = tree.currentId
  if (!nodeId) return
  content.value = v
  appStatus.setSaving()
  try {
    await flush()
    await tree.saveNodeContent(nodeId, v)
    appStatus.setSaved()
    showSaved()
  } catch (error) {
    appStatus.setError()
  }
}

function forceSavePending() {
  void flush()
}

window.addEventListener('blur', forceSavePending)
window.addEventListener('beforeunload', forceSavePending)

onBeforeUnmount(() => {
  window.removeEventListener('blur', forceSavePending)
  window.removeEventListener('beforeunload', forceSavePending)
  forceSavePending()
})

// ── Image upload ──────────────────────────────────────────────
async function onUploadImg(files: File[], callback: (urls: string[]) => void) {
  if (!cfg.cfg.imagesDir) { window.alert(t('notes.images_dir_required')); return }
  try { await invoke('fs_ensure_dir', { path: cfg.cfg.imagesDir! }) } catch {}
  try {
    const urls: string[] = []
    for (const f of files) {
      const buf = new Uint8Array(await f.arrayBuffer())
      const ext = f.name.split('.').pop() || 'png'
      const name = `${Date.now()}_${Math.random().toString(16).slice(2)}.${ext}`
      const p = await join(cfg.cfg.imagesDir!, name)
      await invoke('fs_write_binary', { path: p, bytes: Array.from(buf) })
      urls.push(buildImagePlaceholder(p, cfg.cfg))
    }
    callback(urls)
  } catch (e: any) { window.alert(t('notes.image_upload_failed') + (e?.message || String(e))) }
}

// ── Sanitize local file:// images for Tauri ──────────────────
function sanitizeHtml(html: string) {
  return sanitizeHtmlExt(html, cfg.cfg)
}

const hasNode = computed(() => !!tree.current)
const isTextNode = computed(() => tree.current?.viewType === 'text')
const editorTheme = computed(() => cfg.cfg.theme === 'dark' ? 'dark' : 'light')
const editorFooters: MdEditorFooter[] = ['markdownTotal']


</script>

<template>
  <div class="notes-view">
    <LaySplitPanel :initial="240">
      <template #left>
        <div class="sidebar">
          <NotebookTree />
        </div>
      </template>
      <template #right>
        <div class="content-area">
          <template v-if="hasNode">
            <!-- Toolbar -->
            <div class="content-toolbar">
              <span class="node-title">{{ tree.current!.label }}</span>
              <div class="spacer" />
              <span v-if="saveMsg" class="save-msg">{{ saveMsg }}</span>
            </div>
            <!-- Tags bar -->
            <div v-if="tree.current!.tags?.length" class="tags-bar">
              <span v-for="tag in tree.current!.tags" :key="tag" class="tag-chip">{{ tag }}</span>
            </div>
            <!-- viewType dispatcher -->
            <div class="editor-wrap">
              <textarea
                v-if="isTextNode"
                v-model="content"
                class="plain-editor"
                spellcheck="false"
                @input="onChange(content)"
              />
              <MdEditor
                v-else-if="tree.current!.viewType === 'markdown'"
                editorId="main-editor"
                v-model="content"
                :theme="editorTheme"
                :footers="editorFooters"
                :scroll-auto="false"
                :onUploadImg="onUploadImg"
                :sanitize="sanitizeHtml"
                @onChange="onChange"
                @onSave="onSave"
                :style="{ height: '100%', width: '100%' }"
              />
              <div v-else class="plugin-placeholder">
                Unsupported note type: {{ tree.current!.viewType }}
              </div>
            </div>
          </template>
          <template v-else>
            <div class="empty-state">{{ t('editor.select_node') }}</div>
          </template>
        </div>
      </template>
    </LaySplitPanel>

  </div>
</template>

<style scoped>
.notes-view { display: flex; height: 100%; width: 100%; overflow: hidden; }
.content-area { display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; position: relative; }
.sidebar { display: flex; flex-direction: column; height: 100%; }
.content-toolbar { display: flex; align-items: center; padding: 6px 12px; border-bottom: 1px solid var(--app-border); gap: 8px; min-height: 36px; background: var(--app-surface); }
.node-title { font-weight: 600; font-size: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 300px; color: var(--app-text); }
.spacer { flex: 1; }
.save-msg { font-size: 12px; color: #52c41a; }
.editor-wrap { flex: 1; overflow: hidden;width:100%; height: 100%; position: relative; }
:deep(.md-editor) { width: 100%; height: 100%; max-width: none; border: none; }
:deep(.md-editor-dark) {
  --md-color: var(--app-text);
  --md-hover-color: #ffffff;
  --md-bk-color: var(--app-bg);
  --md-bk-color-outstand: var(--app-surface);
  --md-bk-hover-color: var(--app-hover);
  --md-border-color: var(--app-border);
  --md-border-hover-color: #3c3c3c;
  --md-border-active-color: #6a6a6a;
  --md-scrollbar-bg-color: #1e1e1e;
  --md-scrollbar-thumb-color: #424242;
  --md-scrollbar-thumb-hover-color: #4f4f4f;
  --md-scrollbar-thumb-active-color: #5f5f5f;
}
:deep(.md-editor-dark .md-editor-preview) {
  --md-theme-color: var(--app-text);
  --md-theme-color-hover: var(--app-hover);
  --md-theme-color-hover-inset: var(--app-active);
  --md-theme-border-color: var(--app-border);
  --md-theme-bg-color: var(--app-bg);
  --md-theme-bg-color-inset: var(--app-surface);
  --md-theme-bg-color-scrollbar-track: #1e1e1e;
  --md-theme-bg-color-scrollbar-thumb: #424242;
  --md-theme-bg-color-scrollbar-thumb-hover: #4f4f4f;
  --md-theme-bg-color-scrollbar-thumb-active: #5f5f5f;
}
:deep(.md-editor-dark),
:deep(.md-editor-dark .md-editor-content),
:deep(.md-editor-dark .md-editor-input-wrapper),
:deep(.md-editor-dark .md-editor-preview-wrapper),
:deep(.md-editor-dark .md-editor-preview),
:deep(.md-editor-dark .cm-editor),
:deep(.md-editor-dark .cm-scroller),
:deep(.md-editor-dark .cm-content),
:deep(.md-editor-dark .cm-gutters) {
  background: var(--app-bg);
}
:deep(.md-editor-dark .md-editor-toolbar-wrapper),
:deep(.md-editor-dark .md-editor-footer) {
  color: var(--app-text);
  background: var(--app-surface);
  border-color: var(--app-border);
}
:deep(.md-editor-dark .cm-gutters) {
  color: var(--app-muted);
  border-right-color: var(--app-border);
}
:deep(.md-editor-dark .cm-activeLine),
:deep(.md-editor-dark .cm-activeLineGutter) {
  background: var(--app-surface);
}
:deep(.md-editor-content) { width: 100%; margin: 0 auto; }
.plain-editor { width: 100%; height: 100%; box-sizing: border-box; border: none; outline: none; resize: none; padding: 18px 22px; font-size: 15px; line-height: 1.7; color: var(--app-text); background: var(--app-bg); font-family: inherit; }
.tags-bar { display: flex; flex-wrap: wrap; gap: 6px; padding: 4px 12px; border-bottom: 1px solid var(--app-border); background: var(--app-surface); }
.tag-chip { display: inline-flex; align-items: center; padding: 1px 8px; background: #e8f3ff; color: #1677ff; border-radius: 12px; font-size: 12px; }
.empty-state { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--app-muted); font-size: 14px; }
.plugin-placeholder { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--app-muted); font-size: 13px; }
</style>
