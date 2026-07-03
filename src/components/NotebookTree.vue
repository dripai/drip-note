<script setup lang="ts">
import { computed, onMounted, ref, reactive } from 'vue'
import { Modal } from 'ant-design-vue'
import { useTreeStore } from '../stores/treeStore'
import { useI18n } from 'vue-i18n'
import ContextMenu from './ContextMenu.vue'
import { sqliteProvider } from '../services/storage/sqlite.provider'
import type { TreeNode } from '../services/storage/storage.types'
import IconAdd from '~icons/ri/add-line'
import IconFolderAdd from '~icons/ri/folder-add-line'
import IconEdit from '~icons/ri/edit-line'
import IconSettings from '~icons/ri/settings-3-line'
import IconArrowUp from '~icons/ri/arrow-up-line'
import IconArrowDown from '~icons/ri/arrow-down-line'
import IconDragMove from '~icons/ri/drag-move-2-line'
import IconDelete from '~icons/ri/delete-bin-line'
import IconRenderer from './icons/IconRenderer.vue'
import IconSelect from './icons/IconSelect.vue'

const tree = useTreeStore()
const { t } = useI18n()

// ── Expanded state persistence ─────────────────────────────────
const EXPAND_KEY = 'drip-tree-expanded'
const expandedKeys = ref<string[]>(JSON.parse(localStorage.getItem(EXPAND_KEY) || '[]'))

function onTreeExpand(keys: Array<string | number>) {
  expandedKeys.value = keys.map(String)
  localStorage.setItem(EXPAND_KEY, JSON.stringify(expandedKeys.value))
}

onMounted(async () => {
  await tree.init()
})

// ── Context menu ──────────────────────────────────────────────
const menu = reactive<{ visible: boolean; x: number; y: number; nodeId: string; items: any[] }>({
  visible: false, x: 0, y: 0, nodeId: '', items: [],
})

function closeMenu() { menu.visible = false }

function openNodeMenu(e: MouseEvent, nodeId: string) {
  e.preventDefault(); e.stopPropagation()
  menu.visible = true; menu.x = e.clientX; menu.y = e.clientY; menu.nodeId = nodeId
  const node = tree.flat.find(n => n.id === nodeId)
  const siblings = tree.flat.filter(n => n.parentId === (node?.parentId ?? null)).sort((a, b) => a.sortOrder - b.sortOrder)
  const idx = siblings.findIndex(n => n.id === nodeId)
  menu.items = [
    { label: t('tree.add_child'), key: 'addChild', icon: IconAdd },
    { label: t('tree.rename'), key: 'rename', icon: IconEdit },
    { label: t('tree.properties'), key: 'editIconColor', icon: IconSettings },
    { separator: true },
    ...(idx > 0 ? [{ label: t('tree.move_up'), key: 'moveUp', icon: IconArrowUp }] : []),
    ...(idx < siblings.length - 1 ? [{ label: t('tree.move_down'), key: 'moveDown', icon: IconArrowDown }] : []),
    { label: t('tree.move_to'), key: 'moveTo', icon: IconDragMove },
    { separator: true },
    { label: t('tree.delete'), key: 'delete', icon: IconDelete, color: 'danger' },
  ]
}


function confirmDelete() {
  return new Promise<void>((resolve, reject) => {
    Modal.confirm({
      title: t('tree.delete_title'),
      content: t('tree.delete_confirm'),
      okText: t('common.delete'),
      cancelText: t('common.cancel'),
      okType: 'danger',
      onOk: () => resolve(),
      onCancel: () => reject(new Error('cancelled')),
    })
  })
}

function openPanelMenu(e: MouseEvent) {
  e.preventDefault()
  menu.visible = true; menu.x = e.clientX; menu.y = e.clientY; menu.nodeId = ''
  menu.items = [{ label: t('tree.add_notebook'), key: 'addRoot', icon: IconFolderAdd }]
}

async function onSelectMenu(key: string) {
  const id = menu.nodeId
  closeMenu()
  if (key === 'addRoot') {
    await promptAndAdd(null)
  } else if (key === 'addChild') {
    await promptAndAdd(id)
  } else if (key === 'rename') {
    const node = tree.flat.find(n => n.id === id)
    if (!node) return
    openRenameDialog(node)
  } else if (key === 'delete') {
    const hasChildren = tree.flat.some(n => n.parentId === id)
    if (hasChildren) {
      Modal.warning({ title: t('tree.delete_title'), content: t('tree.delete_has_children'), okText: t('common.ok') })
      return
    }
    try {
      await confirmDelete()
      const deleted = await tree.deleteNode(id)
      if (!deleted) {
        Modal.warning({ title: t('tree.delete_title'), content: t('tree.delete_has_children'), okText: t('common.ok') })
      }
    } catch {}
  } else if (key === 'moveUp' || key === 'moveDown') {
    const node = tree.flat.find(n => n.id === id)
    if (!node) return
    const siblings = tree.flat.filter(n => n.parentId === node.parentId).sort((a, b) => a.sortOrder - b.sortOrder)
    const idx = siblings.findIndex(n => n.id === id)
    const swapIdx = key === 'moveUp' ? idx - 1 : idx + 1
    if (swapIdx < 0 || swapIdx >= siblings.length) return
    const ids = siblings.map(n => n.id)
    ids.splice(idx, 1)
    ids.splice(swapIdx, 0, id)
    await tree.reorderSiblings(node.parentId, ids)
  } else if (key === 'editIconColor') {
    const node = tree.flat.find(n => n.id === id)
    if (!node) return
    iconColorPicker.nodeId = id
    iconColorPicker.icon = node.icon ?? ''
    iconColorPicker.tags = node.tags ? [...node.tags] : []
    iconColorPicker.tagInput = ''
    iconColorPicker.visible = true
  } else if (key === 'moveTo') {
    movePicker.nodeId = id
    movePicker.visible = true
  }
}

// ── Rename dialog ───────────────────────────────────────────────
const renameDialog = reactive<{ visible: boolean; nodeId: string; label: string }>({
  visible: false, nodeId: '', label: '',
})
const renameInputRef = ref<any>(null)

function openRenameDialog(node: TreeNode) {
  renameDialog.nodeId = node.id
  renameDialog.label = node.label
  renameDialog.visible = true
}

async function confirmRename() {
  const label = renameDialog.label.trim()
  if (!label) return
  renameDialog.visible = false
  await tree.saveLabel(renameDialog.nodeId, label)
}

// ── Move To picker ─────────────────────────────────────────────
const movePicker = reactive<{ visible: boolean; nodeId: string; selectedId: string | null }>({
  visible: false, nodeId: '', selectedId: null,
})

function moveToOptions(nodeId: string) {
  const descendants = new Set<string>()
  const collect = (id: string) => {
    descendants.add(id)
    tree.flat.filter(n => n.parentId === id).forEach(n => collect(n.id))
  }
  collect(nodeId)
  const result: Array<{ id: string; label: string; disabled: boolean }> = []
  const walk = (parentId: string | null) => {
    const children = tree.flat.filter(n => n.parentId === parentId).sort((a, b) => a.sortOrder - b.sortOrder)
    for (const n of children) {
      result.push({ id: n.id, label: n.label, disabled: descendants.has(n.id) })
      walk(n.id)
    }
  }
  walk(null)
  return result
}

function moveToDepth(id: string): number {
  let depth = 0
  let node = tree.flat.find(n => n.id === id)
  while (node?.parentId) { depth++; node = tree.flat.find(n => n.id === node!.parentId) }
  return depth
}

async function confirmMoveTo() {
  const { nodeId, selectedId } = movePicker
  movePicker.visible = false
  await tree.moveNode(nodeId, selectedId)
}

// ── Add node ───────────────────────────────────────────────────
const viewTypePicker = reactive<{ visible: boolean; parentId: string | null; label: string; viewType: string; title: string }>({
  visible: false, parentId: null, label: '', viewType: 'text', title: '',
})
const addNodeInputRef = ref<HTMLInputElement | null>(null)
const viewTypeOptions = [
  { value: 'text', labelKey: 'tree.vt_text', icon: 'T', descKey: 'tree.vt_text_desc' },
  { value: 'markdown', labelKey: 'tree.vt_markdown', icon: 'M', descKey: 'tree.vt_markdown_desc' },
]

function promptAndAdd(parentId: string | null, viewType = 'text') {
  viewTypePicker.parentId = parentId
  viewTypePicker.label = ''
  viewTypePicker.viewType = viewType
  viewTypePicker.title = parentId ? t('tree.add_child') : t('tree.add_notebook')
  viewTypePicker.visible = true
}

function onCreateRootMenu({ key }: { key: string }) {
  if (key === 'text' || key === 'markdown') promptAndAdd(null, key)
}

async function confirmViewType() {
  const label = viewTypePicker.label.trim()
  if (!label) return
  viewTypePicker.visible = false
  const { parentId, viewType } = viewTypePicker
  const newId = parentId
    ? await tree.addChild(parentId, label, viewType)
    : await tree.addRoot(label, viewType)
  await tree.selectNode(newId)
}

// ── Node property picker ────────────────────────────────────────
const iconColorPicker = reactive<{ visible: boolean; nodeId: string; icon: string | undefined; tagInput: string; tags: string[] }>({
  visible: false, nodeId: '', icon: undefined, tagInput: '', tags: [],
})

function addTag() {
  const tag = iconColorPicker.tagInput.trim()
  if (tag && !iconColorPicker.tags.includes(tag)) iconColorPicker.tags.push(tag)
  iconColorPicker.tagInput = ''
}

async function confirmIconColor() {
  iconColorPicker.visible = false
  await tree.saveNodeProperties(iconColorPicker.nodeId, iconColorPicker.icon || undefined, iconColorPicker.tags.length ? iconColorPicker.tags : undefined)
}

// ── Search ────────────────────────────────────────────────────
const searchQuery = ref('')
const searchResults = ref<Array<{ id: string; label: string; snippet: string }>>([])
const isSearching = computed(() => !!searchQuery.value.trim())
let searchTimer: ReturnType<typeof setTimeout> | null = null

function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  const q = searchQuery.value.trim()
  if (!q) { searchResults.value = []; return }
  searchTimer = setTimeout(async () => {
    searchResults.value = await sqliteProvider.search(q)
  }, 300)
}

async function selectSearchResult(id: string) {
  searchQuery.value = ''
  searchResults.value = []
  await tree.selectNode(id)
}

function nodePath(id: string) {
  const parts: string[] = []
  let node = tree.flat.find(n => n.id === id)
  while (node) {
    parts.unshift(node.label)
    node = node.parentId ? tree.flat.find(n => n.id === node!.parentId) : undefined
  }
  return parts.join(' / ')
}

// ── el-tree config ─────────────────────────────────────────────
const treeFieldNames = { children: 'children', title: 'label', key: 'id' }

function onNodeClick(node: TreeNode) {
  tree.selectNode(node.id)
}

function onNodeTitleContextMenu(e: MouseEvent, node: TreeNode) {
  e.preventDefault()
  e.stopPropagation()
  openNodeMenu(e, node.id)
}
</script>

<template>
  <div class="nb-tree" @contextmenu.prevent="openPanelMenu($event)">
    <div class="search-box">
      <input
        v-model="searchQuery"
        :placeholder="t('editor.search_placeholder')"
        class="search-input"
        @input="onSearchInput"
      />
      <a-dropdown :trigger="['click']">
        <button class="icon-btn" :title="t('tree.add_notebook')" @click.stop><IconAdd /></button>
        <template #overlay>
          <a-menu @click="onCreateRootMenu">
            <a-menu-item key="text">
              <span class="create-type-mark">T</span>
              <span>{{ t('tree.vt_text') }}</span>
            </a-menu-item>
            <a-menu-item key="markdown">
              <span class="create-type-mark">M</span>
              <span>{{ t('tree.vt_markdown') }}</span>
            </a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>
    </div>
    <div class="tree-body">
      <div v-if="isSearching" class="search-mode">
        <div class="search-mode-title">{{ t('editor.search_results') }}</div>
        <div v-if="searchResults.length" class="search-result-list">
          <div v-for="r in searchResults" :key="r.id" class="search-item" @click="selectSearchResult(r.id)">
            <div class="search-item-label">{{ r.label }}</div>
            <div class="search-item-path">{{ nodePath(r.id) }}</div>
            <div class="search-item-snippet" v-html="r.snippet" />
          </div>
        </div>
        <div v-else class="search-empty">{{ t('editor.search_no_result') }}</div>
      </div>
      <a-tree
        v-else-if="tree.roots.length"
        :tree-data="tree.roots"
        :field-names="treeFieldNames"
        :selected-keys="tree.currentId ? [tree.currentId] : []"
        :expanded-keys="expandedKeys"
        :show-line="{ showLeafIcon: false }"
        block-node
        @select="(_: unknown, info: any) => onNodeClick(info.node as unknown as TreeNode)"
        @expand="onTreeExpand"
      >
        <template #title="node">
          <span
            class="custom-node"
            @contextmenu="onNodeTitleContextMenu($event, node as TreeNode)"
          >
            <span v-if="node.icon" class="node-icon"><IconRenderer :icon="node.icon" /></span>
            <span class="node-label-text">{{ node.label }}</span>
          </span>
        </template>
      </a-tree>
      <div v-else class="empty-hint">{{ t('tree.empty_hint') }}</div>
    </div>
    <ContextMenu :visible="menu.visible" :x="menu.x" :y="menu.y" :items="menu.items" @select="onSelectMenu" @close="closeMenu" />

    <!-- Node properties dialog -->
    <a-modal v-model:open="iconColorPicker.visible" :title="t('tree.properties')" width="420px">
      <div class="icon-color-form">
        <div class="property-row">
          <span class="ic-label">{{ t('tree.icon') }}</span>
          <IconSelect v-model:value="iconColorPicker.icon" />
        </div>
        <div class="property-row property-row-stack">
          <span class="ic-label">{{ t('tree.tags') }}</span>
          <div class="tag-input-row">
            <input v-model="iconColorPicker.tagInput" class="tag-input" :placeholder="t('tree.tag_placeholder')" @keyup.enter="addTag" />
            <button class="tag-add-btn" @click="addTag">+</button>
          </div>
          <div class="tag-list">
            <span v-for="tag in iconColorPicker.tags" :key="tag" class="tag-chip">
              {{ tag }}<span class="tag-remove" @click="iconColorPicker.tags = iconColorPicker.tags.filter(x => x !== tag)">✕</span>
            </span>
          </div>
        </div>
      </div>
      <template #footer>
        <a-button @click="iconColorPicker.visible = false">{{ t('common.cancel') }}</a-button>
        <a-button type="primary" @click="confirmIconColor">{{ t('common.ok') }}</a-button>
      </template>
    </a-modal>

    <!-- Move To dialog -->
    <a-modal v-model:open="movePicker.visible" :title="t('tree.move_to_title')" width="320px">
      <div class="move-to-list">
        <div class="move-to-item" :class="{ selected: movePicker.selectedId === null }" @click="movePicker.selectedId = null">{{ t('tree.move_to_root') }}</div>
        <div v-for="n in moveToOptions(movePicker.nodeId)" :key="n.id" class="move-to-item" :class="{ selected: movePicker.selectedId === n.id, disabled: n.disabled }" @click="!n.disabled && (movePicker.selectedId = n.id)">{{ '　'.repeat(moveToDepth(n.id)) }}{{ n.label }}</div>
      </div>
      <template #footer>
        <a-button @click="movePicker.visible = false">{{ t('common.cancel') }}</a-button>
        <a-button type="primary" @click="confirmMoveTo">{{ t('common.ok') }}</a-button>
      </template>
    </a-modal>

    <!-- Rename dialog -->
    <a-modal v-model:open="renameDialog.visible" :title="t('tree.rename')" width="360px" @after-open-change="(open: boolean) => open && renameInputRef?.focus()">
      <a-input
        ref="renameInputRef"
        v-model:value="renameDialog.label"
        :placeholder="t('tree.enter_name')"
        allow-clear
        @press-enter="confirmRename"
      />
      <template #footer>
        <a-button @click="renameDialog.visible = false">{{ t('common.cancel') }}</a-button>
        <a-button type="primary" :disabled="!renameDialog.label.trim()" @click="confirmRename">{{ t('common.ok') }}</a-button>
      </template>
    </a-modal>

    <!-- Add node dialog -->
    <a-modal v-model:open="viewTypePicker.visible" :title="viewTypePicker.title" width="360px" @after-open-change="(open: boolean) => open && addNodeInputRef?.focus()">
      <div class="add-node-form">
        <input ref="addNodeInputRef" v-model="viewTypePicker.label" :placeholder="t('tree.enter_name')" class="add-node-input" @keyup.enter="confirmViewType" />
        <div class="vt-options">
          <div v-for="opt in viewTypeOptions" :key="opt.value" class="vt-option" :class="{ selected: viewTypePicker.viewType === opt.value }" @click="viewTypePicker.viewType = opt.value">
            <span class="vt-icon">{{ opt.icon }}</span>
            <span class="vt-label">{{ t(opt.labelKey) }}</span>
            <span class="vt-desc">{{ t(opt.descKey) }}</span>
          </div>
        </div>
      </div>
      <template #footer>
        <a-button @click="viewTypePicker.visible = false">{{ t('common.cancel') }}</a-button>
        <a-button type="primary" :disabled="!viewTypePicker.label.trim()" @click="confirmViewType">{{ t('common.ok') }}</a-button>
      </template>
    </a-modal>
  </div>
</template>

<style scoped>
.nb-tree { height: 100%; display: flex; flex-direction: column; user-select: none; color: var(--app-text); background: var(--app-panel); }
.icon-btn { width: 24px; height: 24px; border: none; background: none; cursor: pointer; padding: 4px; border-radius: 4px; color: var(--app-muted); font-size: 16px; display: flex; align-items: center; justify-content: center; }
.icon-btn:hover { background: var(--app-hover); color: #1677ff; }
.create-type-mark { display: inline-flex; width: 18px; height: 18px; align-items: center; justify-content: center; margin-right: 8px; color: var(--app-muted); font-size: 11px; font-weight: 700; border: 1px solid var(--app-border); border-radius: 3px; }
.search-box { position: relative; display: flex; align-items: center; gap: 6px; padding: 7px 8px; border-bottom: 1px solid var(--app-border); }
.search-input { min-width: 0; width: 100%; height: 28px; box-sizing: border-box; flex: 1; padding: 4px 8px; font-size: 12px; color: var(--app-text); background: var(--app-surface); border: 1px solid var(--app-border); border-radius: 4px; outline: none; }
.search-input:focus { border-color: #1677ff; }
.search-mode { padding: 4px 6px 10px; }
.search-mode-title { padding: 5px 6px; color: var(--app-muted); font-size: 11px; font-weight: 600; text-transform: uppercase; }
.search-result-list { display: flex; flex-direction: column; gap: 2px; }
.search-item { padding: 7px 8px; cursor: pointer; border-radius: 4px; }
.search-item:hover { background: var(--app-hover); }
.search-item-label { overflow: hidden; color: var(--app-text); font-size: 13px; font-weight: 600; text-overflow: ellipsis; white-space: nowrap; }
.search-item-path { overflow: hidden; margin-top: 2px; color: var(--app-muted); font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }
.search-item-snippet { margin-top: 3px; color: #888; font-size: 12px; line-height: 1.35; }
.search-empty { padding: 10px 12px; color: var(--app-muted); font-size: 12px; }
.tree-body { flex: 1; overflow-y: auto; padding: 2px 0; }
.empty-hint { padding: 16px; text-align: center; color: #bbb; font-size: 13px; }

/* tree overrides */
:deep(.ant-tree) { color: var(--app-text); background: transparent; font-size: 13px; }
:deep(.ant-tree-node-content-wrapper) { height: 28px; border-radius: 4px; color: var(--app-text); }
:deep(.ant-tree-node-content-wrapper:hover) { background: var(--app-hover); }
:deep(.ant-tree .ant-tree-node-selected) { background: var(--app-active); color: var(--app-active-text); font-weight: 600; }
:deep(.ant-tree-switcher) { color: #888; font-size: 13px; }
:deep(.ant-tree .ant-tree-node-selected .node-icon) { color: var(--app-active-text); }

.custom-node { display: flex; align-items: center; gap: 4px; flex: 1; overflow: hidden; }
.node-icon { display: inline-flex; align-items: center; justify-content: center; width: 16px; font-size: 14px; flex-shrink: 0; color: #64748b; }
.node-label-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.add-node-form { display: flex; flex-direction: column; gap: 12px; }
.add-node-input { width: 100%; box-sizing: border-box; padding: 7px 10px; font-size: 13px; color: var(--app-text); background: var(--app-surface); border: 1px solid var(--app-border); border-radius: 4px; outline: none; }
.add-node-input:focus { border-color: #1677ff; }
.vt-options { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; }
.vt-option { display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 10px 6px; border: 1px solid #ddd; border-radius: 6px; cursor: pointer; text-align: center; }
.vt-option:hover { border-color: #1677ff; background: #f5f9ff; }
.vt-option.selected { border-color: #1677ff; background: #e8f3ff; }
.vt-icon { font-size: 22px; line-height: 1; }
.vt-label { font-size: 12px; font-weight: 600; color: #333; }
.vt-desc { font-size: 11px; color: #999; line-height: 1.3; }
.move-to-list { max-height: 300px; overflow-y: auto; border: 1px solid #eee; border-radius: 4px; }
.move-to-item { padding: 7px 12px; font-size: 13px; cursor: pointer; color: #333; }
.move-to-item:hover:not(.disabled) { background: #f5f5f5; }
.move-to-item.selected { background: #e8f3ff; color: #1677ff; font-weight: 600; }
.move-to-item.disabled { color: #bbb; cursor: not-allowed; }
.icon-color-form { display: flex; flex-direction: column; gap: 16px; }
.property-row { display: grid; grid-template-columns: 48px minmax(0, 1fr); align-items: center; gap: 10px; }
.property-row-stack { align-items: flex-start; }
.ic-label { font-size: 13px; color: #555; line-height: 32px; }
.tag-input-row { display: flex; gap: 6px; width: 100%; }
.tag-input { flex: 1; padding: 4px 8px; font-size: 12px; border: 1px solid #ddd; border-radius: 4px; outline: none; }
.tag-input:focus { border-color: #1677ff; }
.tag-add-btn { padding: 4px 10px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; background: #fafafa; font-size: 14px; }
.tag-add-btn:hover { border-color: #1677ff; color: #1677ff; }
.tag-list { display: flex; flex-wrap: wrap; gap: 6px; min-height: 24px; }
.tag-chip { display: inline-flex; align-items: center; gap: 4px; padding: 2px 8px; background: #e8f3ff; color: #1677ff; border-radius: 12px; font-size: 12px; }
.tag-remove { cursor: pointer; font-size: 10px; color: #aaa; }
.tag-remove:hover { color: #f5222d; }
</style>
