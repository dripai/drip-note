<script setup lang="ts">
import type { TreeNode } from '../services/storage/storage.types'

const props = defineProps<{
  node: TreeNode
  currentId: string | null
  dragOverId: string | null
  depth?: number
}>()

const emit = defineEmits<{
  select: [id: string]
  contextmenu: [payload: { e: MouseEvent; id: string }]
  dragstart: [payload: { e: DragEvent; id: string }]
  dragover: [payload: { e: DragEvent; id: string }]
  dragleave: []
  drop: [payload: { e: DragEvent; id: string; parentId: string | null }]
  dragend: []
}>()

const depth = props.depth ?? 0
const expanded = defineModel<boolean>('expanded', { default: true })

// Vertical guide line x position (from left edge of .node-children container):
// node-row has margin-left:4px + border:3px = 7px offset
// toggle center = 7 + paddingLeft + 7(half toggle) = 7 + (10 + depth*16) + 7 = 24 + depth*16
function guideLeft(d: number) { return `${24 + d * 16}px` }
</script>

<template>
  <div class="tree-node">
    <div
      class="node-row"
      :class="{ active: currentId === node.id, 'drag-over': dragOverId === node.id, 'child-node': depth > 0 }"
      :style="{ paddingLeft: `${10 + depth * 16}px`, borderLeft: node.color ? `3px solid ${node.color}` : '3px solid transparent' }"
      draggable="true"
      @click="emit('select', node.id)"
      @contextmenu.prevent.stop="emit('contextmenu', { e: $event, id: node.id })"
      @dragstart.stop="emit('dragstart', { e: $event, id: node.id })"
      @dragover.prevent.stop="emit('dragover', { e: $event, id: node.id })"
      @dragleave.stop="emit('dragleave')"
      @drop.prevent.stop="emit('drop', { e: $event, id: node.id, parentId: node.parentId })"
      @dragend.stop="emit('dragend')"
    >
      <span class="toggle" @click.stop="expanded = !expanded">
        <template v-if="node.children && node.children.length">
          {{ expanded ? '▾' : '▸' }}
        </template>
        <template v-else>
          <span class="leaf-dot">·</span>
        </template>
      </span>
      <span v-if="node.icon" class="node-icon">{{ node.icon }}</span>
      <span class="node-label">{{ node.label }}</span>
    </div>
    <div
      v-if="expanded && node.children && node.children.length"
      class="node-children"
      :style="{ '--guide-x': guideLeft(depth) }"
    >
      <TreeNodeItem
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :current-id="currentId"
        :drag-over-id="dragOverId"
        :depth="depth + 1"
        @select="emit('select', $event)"
        @contextmenu="emit('contextmenu', $event)"
        @dragstart="emit('dragstart', $event)"
        @dragover="emit('dragover', $event)"
        @dragleave="emit('dragleave')"
        @drop="emit('drop', $event)"
        @dragend="emit('dragend')"
      />
    </div>
  </div>
</template>

<style scoped>
.tree-node { position: relative; }
.node-row {
  position: relative;
  display: flex; align-items: center; gap: 4px;
  padding-top: 5px; padding-right: 8px; padding-bottom: 5px;
  cursor: pointer; border-radius: 4px; margin: 1px 4px;
  font-size: 13px; color: #333;
}
.node-row:hover { background: #f5f5f5; }
.node-row.active { background: #e8f3ff; color: #1677ff; font-weight: 600; }
.node-row.drag-over { background: #d0e8ff; outline: 1px dashed #1677ff; }
.toggle { width: 14px; text-align: center; font-size: 13px; color: #888; flex-shrink: 0; }
.leaf-dot { color: #bbb; }
.node-label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.node-icon { font-size: 14px; line-height: 1; flex-shrink: 0; }

/* vertical guide line */
.node-children {
  position: relative;
}
.node-children::before {
  content: '';
  position: absolute;
  left: var(--guide-x);
  top: 0;
  bottom: 6px;
  border-left: 1px dashed #bbb;
  pointer-events: none;
}

/* horizontal connector: from vertical line to node toggle */
.child-node::before {
  content: '';
  position: absolute;
  left: calc(var(--guide-x) - 7px);
  top: 50%;
  width: 9px;
  border-top: 1px dashed #bbb;
  pointer-events: none;
}
</style>
