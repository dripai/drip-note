<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue'
// Add icon and color support
interface MenuItem {
  label: string
  key: string
  icon?: any // Component or string
  color?: string
  disabled?: boolean
  separator?: boolean
}

const props = defineProps<{ 
  visible: boolean
  x: number
  y: number
  items: MenuItem[] 
}>()

const emit = defineEmits<{ (e: 'select', key: string): void; (e: 'close'): void }>()
const root = ref<HTMLElement | null>(null)

function select(item: MenuItem) { 
  if (item.disabled || item.separator) return
  emit('select', item.key)
  emit('close') 
}

function onGlobalDown(e: MouseEvent) { 
  if (!root.value) return
  // If clicking outside, close
  if (props.visible && !root.value.contains(e.target as Node)) {
    emit('close')
  }
}

onMounted(() => { window.addEventListener('mousedown', onGlobalDown) })
onBeforeUnmount(() => { window.removeEventListener('mousedown', onGlobalDown) })
</script>

<template>
  <transition name="fade-scale">
    <div 
      v-if="visible" 
      ref="root" 
      class="context-menu" 
      :style="{ left: x + 'px', top: y + 'px' }" 
      @contextmenu.prevent
    >
      <ul class="menu-list">
        <template v-for="(it, idx) in items" :key="it.key || idx">
          <li v-if="it.separator" class="menu-separator" />
          <li 
            v-else
            class="menu-item" 
            :class="{ disabled: it.disabled, danger: it.color === 'danger' }"
            @click="select(it)"
          >
            <div class="menu-icon">
              <component :is="it.icon" v-if="it.icon" />
            </div>
            <span class="menu-label">{{ it.label }}</span>
          </li>
        </template>
      </ul>
    </div>
  </transition>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 9999;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(0, 0, 0, 0.08);
  box-shadow: 
    0 4px 6px -1px rgba(0, 0, 0, 0.1), 
    0 2px 4px -1px rgba(0, 0, 0, 0.06),
    0 12px 24px -4px rgba(0, 0, 0, 0.1);
  border-radius: 12px;
  min-width: 180px;
  padding: 6px;
  transform-origin: top left;
}

.menu-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 8px;
  transition: all 0.2s;
  color: #374151;
  font-size: 13px;
  font-weight: 500;
  user-select: none;
}

.menu-item:hover:not(.disabled) {
  background: #f3f4f6;
  color: #111827;
}

.menu-item.danger {
  color: #ef4444;
}

.menu-item.danger:hover:not(.disabled) {
  background: #fef2f2;
  color: #dc2626;
}

.menu-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.menu-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  margin-right: 10px;
  font-size: 16px;
  color: inherit;
  opacity: 0.7;
}

.menu-label {
  flex: 1;
}

.menu-separator {
  height: 1px;
  background: #e5e7eb;
  margin: 4px 8px;
}

/* Transitions */
.fade-scale-enter-active,
.fade-scale-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.fade-scale-enter-from,
.fade-scale-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
