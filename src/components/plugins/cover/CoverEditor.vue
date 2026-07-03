<script setup lang="ts">
import { ref, reactive, nextTick } from 'vue'
import Moveable from 'vue3-moveable'
import ContextMenu from '@/components/ContextMenu.vue'
import IconText from '~icons/ri/text'
import IconStickyNote from '~icons/ri/sticky-note-line'
import IconImage from '~icons/ri/image-line'

defineProps<{
  nodeId: string
}>()

interface CanvasItem {
  id: string
  type: 'text' | 'image' | 'sticky'
  x: number
  y: number
  width: number
  height: number
  content: string
  style: Record<string, any>
  zIndex: number
}

const items = ref<CanvasItem[]>([])
const selectedId = ref<string | null>(null)
const moveableRef = ref<any>(null)
const canvasRef = ref<HTMLElement | null>(null)

// Context Menu State
const menu = reactive({
  visible: false,
  x: 0,
  y: 0,
  items: [
    { label: '添加文本', key: 'add-text', icon: IconText },
    { label: '添加便利贴', key: 'add-sticky', icon: IconStickyNote },
    { label: '添加图片', key: 'add-image', icon: IconImage },
  ]
})

// Add new item
function addItem(type: CanvasItem['type'], x: number, y: number) {
  const id = `item-${Date.now()}`
  const zIndex = items.value.length + 1
  
  let newItem: CanvasItem = {
    id,
    type,
    x,
    y,
    width: 200,
    height: 100,
    content: '双击编辑内容',
    style: {},
    zIndex
  }

  if (type === 'sticky') {
    newItem.style = { backgroundColor: '#fff740', padding: '10px', boxShadow: '2px 2px 5px rgba(0,0,0,0.1)' }
    newItem.width = 150
    newItem.height = 150
    newItem.content = '便利贴'
  } else if (type === 'text') {
    newItem.style = { fontSize: '16px', color: '#333' }
    newItem.height = 40
    newItem.content = '请输入文本'
  } else if (type === 'image') {
    newItem.content = 'https://placehold.co/200x150'
    newItem.width = 200
    newItem.height = 150
  }

  items.value.push(newItem)
  selectedId.value = id
  nextTick(() => {
    moveableRef.value?.updateRect()
  })
}

// Interaction Handlers
function onCanvasContextMenu(e: MouseEvent) {
  e.preventDefault()
  menu.x = e.clientX
  menu.y = e.clientY
  menu.visible = true
}

function onMenuSelect(key: string) {
  const rect = canvasRef.value?.getBoundingClientRect()
  const x = menu.x - (rect?.left || 0)
  const y = menu.y - (rect?.top || 0)

  if (key === 'add-text') addItem('text', x, y)
  else if (key === 'add-sticky') addItem('sticky', x, y)
  else if (key === 'add-image') addItem('image', x, y)
  
  menu.visible = false
}

function onSelect(e: MouseEvent, id: string) {
  e.stopPropagation()
  selectedId.value = id
}

function onCanvasClick() {
  selectedId.value = null
  menu.visible = false
}

// Moveable Events
function onDrag({ target, transform }: any) {
  target.style.transform = transform
}
function onDragEnd({ target, isDrag }: any) {
    if (isDrag) {
       const id = target.dataset.id
       const item = items.value.find(i => i.id === id)
       if (item) {
           // Parse transform to update x/y in data model if needed
           // For simplicity in MVP, we rely on DOM transform style
       }
    }
}
function onResize({ target, width, height, drag }: any) {
  target.style.width = `${width}px`
  target.style.height = `${height}px`
  target.style.transform = drag.transform
}
function onRotate({ target, drag }: any) {
  target.style.transform = drag.transform
}

// AI Drop Handler
function onDrop(e: DragEvent) {
  e.preventDefault()
  const toolId = e.dataTransfer?.getData('application/drip-ai-tool')
  if (toolId) {
    const rect = canvasRef.value?.getBoundingClientRect()
    const x = e.clientX - (rect?.left || 0)
    const y = e.clientY - (rect?.top || 0)
    
    if (toolId === 'image') {
        addItem('image', x, y)
    } else if (toolId === 'chat') {
        const id = `ai-chat-${Date.now()}`
        items.value.push({
            id, type: 'text', x, y, width: 240, height: 120, zIndex: items.value.length + 1,
            content: '🤖 AI 对话: 让我帮你生成什么？',
            style: { border: '1px solid #409eff', borderRadius: '8px', padding: '10px', background: '#ecf5ff' }
        })
        selectedId.value = id
    } else {
        addItem('text', x, y) // Default fallback
    }
  }
}
</script>

<template>
  <div 
    ref="canvasRef" 
    class="magic-canvas" 
    @contextmenu="onCanvasContextMenu"
    @click="onCanvasClick"
    @dragover.prevent
    @drop="onDrop"
  >
    <div v-if="items.length === 0" class="empty-state">
      <h2>Magic Canvas (魔力布)</h2>
      <p>拖拽 AI 工具条图标，或右键添加卡片</p>
    </div>

    <div
      v-for="item in items"
      :key="item.id"
      :data-id="item.id"
      class="canvas-item"
      :class="{ selected: selectedId === item.id }"
      :style="{
        left: `${item.x}px`,
        top: `${item.y}px`,
        width: `${item.width}px`,
        height: `${item.height}px`,
        zIndex: item.zIndex,
        ...item.style
      }"
      @click="onSelect($event, item.id)"
    >
      <img v-if="item.type === 'image'" :src="item.content" class="item-content-img" draggable="false" />
      <div v-else class="item-content-text">{{ item.content }}</div>
    </div>

    <Moveable
      v-if="selectedId"
      ref="moveableRef"
      :target="'.canvas-item.selected'"
      :draggable="true"
      :resizable="true"
      :rotatable="true"
      :snappable="true"
      :bounds="{ left: 0, top: 0, bottom: 0, right: 0, position: 'css' }"
      @drag="onDrag"
      @dragEnd="onDragEnd"
      @resize="onResize"
      @rotate="onRotate"
    />

    <ContextMenu 
      :visible="menu.visible" 
      :x="menu.x" 
      :y="menu.y" 
      :items="menu.items" 
      @select="onMenuSelect" 
      @close="menu.visible = false" 
    />
  </div>
</template>

<style scoped>
.magic-canvas {
  width: 100%;
  height: 100%;
  background-color: #f5f5f5;
  background-image: radial-gradient(#ddd 1px, transparent 1px);
  background-size: 20px 20px;
  position: relative;
  overflow: hidden;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #999;
  pointer-events: none;
}

.canvas-item {
  position: absolute;
  box-sizing: border-box;
  cursor: pointer;
  user-select: none;
}

.canvas-item.selected {
  outline: 1px solid #409eff;
}

.item-content-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  pointer-events: none;
}

.item-content-text {
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
