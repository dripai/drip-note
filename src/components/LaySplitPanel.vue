<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps<{ initial?: number, direction?: 'horizontal' | 'vertical' }>()
const primarySize = ref(props.initial ?? 260)
const isDragging = ref(false)
let startX = 0
let startY = 0
let startS = 0
function onMouseDown(e: MouseEvent) { e.preventDefault(); isDragging.value = true; startX = e.clientX; startY = e.clientY; startS = primarySize.value; window.addEventListener('mousemove', onMouseMove); window.addEventListener('mouseup', onMouseUp) }
function onMouseMove(e: MouseEvent) {
  if (!isDragging.value) return
  if ((props.direction ?? 'horizontal') === 'horizontal') {
    const dx = e.clientX - startX
    primarySize.value = Math.max(160, startS + dx)
  } else {
    const dy = e.clientY - startY
    primarySize.value = Math.max(120, startS + dy)
  }
}
function onMouseUp() { isDragging.value = false; window.removeEventListener('mousemove', onMouseMove); window.removeEventListener('mouseup', onMouseUp) }
</script>

<template>
  <div class="split" :class="[ (props.direction ?? 'horizontal'), { dragging: isDragging } ]">
    <div class="left" :style="(props.direction ?? 'horizontal') === 'horizontal' ? { width: primarySize + 'px' } : { height: primarySize + 'px' }">
      <slot name="left" />
    </div>
    <div class="gutter" role="separator" @mousedown="onMouseDown" />
    <div class="right">
      <slot name="right" />
    </div>
  </div>
</template>

<style scoped>
.split { display:flex; height:100%; width: 100%; background: var(--app-bg); }
.split.vertical { flex-direction: column; }
.split.dragging, .split.dragging * { user-select: none }
.left { height:100%; overflow:auto; background: var(--app-panel); }
.split.vertical .left { width:100%; height:auto; border-right:none; border-bottom:0 }
.gutter { position: relative; flex: 0 0 8px; width:8px; cursor: col-resize; background: var(--app-split-bg); }
.gutter::before { position: absolute; top: 0; bottom: 0; left: 50%; width: 1px; content: ''; background: var(--app-split-line); transform: translateX(-50%); }
.gutter::after { position: absolute; top: 50%; left: 50%; width: 3px; height: 120px; content: ''; background: var(--app-split-thumb); border-radius: 2px; opacity: 0; transform: translate(-50%, -50%); transition: opacity .12s ease; }
.gutter:hover::after, .split.dragging .gutter::after { opacity: 1; }
.split.vertical .gutter { width:100%; height:8px; flex-basis: 8px; cursor: row-resize }
.split.vertical .gutter::before { top: 50%; right: 0; bottom: auto; left: 0; width: auto; height: 1px; transform: translateY(-50%); }
.split.vertical .gutter::after { width: 120px; height: 3px; }
.right { flex:1; height:100%; overflow:auto; background: var(--app-bg); }
.split.vertical .right { width:100%; height:auto; flex:1 }
</style>
