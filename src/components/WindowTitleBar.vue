<script setup lang="ts">
import { computed } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useConfigStore } from '@/stores/configStore'
import { useAppStatusStore } from '@/stores/appStatusStore'
import IconSun from '~icons/ri/sun-line'
import IconMoon from '~icons/ri/moon-line'
import IconCloud from '~icons/ri/cloud-line'
import IconCloudOff from '~icons/ri/cloud-off-line'
import IconRefresh from '~icons/ri/refresh-line'
import IconSubtract from '~icons/ri/subtract-line'
import IconCheckbox from '~icons/ri/checkbox-blank-line'
import IconClose from '~icons/ri/close-line'

const cfg = useConfigStore()
const appStatus = useAppStatusStore()
const appWindow = getCurrentWindow()

const platformClass = computed(() => {
  const platform = navigator.platform.toLowerCase()
  if (platform.includes('mac')) return 'titlebar-macos'
  if (platform.includes('linux')) return 'titlebar-linux'
  return 'titlebar-windows'
})

const isDark = computed(() => cfg.cfg.theme === 'dark')
const saveStatusTitle = computed(() => {
  if (appStatus.saveStatus === 'saving') return '保存中'
  if (appStatus.saveStatus === 'error') return '保存失败，点击重试'
  if (appStatus.saveStatus === 'saved') return '已保存，点击立即保存'
  return '待同步，点击立即保存'
})

async function toggleTheme() {
  await cfg.setTheme(isDark.value ? 'light' : 'dark')
}

function requestSave() {
  appStatus.requestSave()
}

function minimizeWindow() {
  void appWindow.minimize()
}

function toggleMaximizeWindow() {
  void appWindow.toggleMaximize()
}

function closeWindow() {
  void appWindow.close()
}

function startDrag(event: MouseEvent) {
  if (event.detail > 1) return
  void appWindow.startDragging()
}
</script>

<template>
  <header class="window-titlebar" :class="platformClass">
    <div class="titlebar-drag-region" data-tauri-drag-region @mousedown="startDrag" @dblclick="toggleMaximizeWindow">
      <div class="app-brand" data-tauri-drag-region>
        <span class="app-logo" data-tauri-drag-region>滴</span>
        <span class="app-title" data-tauri-drag-region>Drip Note</span>
      </div>
    </div>

    <div class="titlebar-toolbar">
      <button class="titlebar-icon-btn" type="button" :title="isDark ? '切换浅色主题' : '切换深色主题'" @click="toggleTheme">
        <IconSun v-if="isDark" />
        <IconMoon v-else />
      </button>
      <button class="titlebar-icon-btn status-btn" :class="`status-${appStatus.saveStatus}`" type="button" :title="saveStatusTitle" @click="requestSave">
        <IconRefresh v-if="appStatus.saveStatus === 'saving'" class="spin-icon" />
        <IconCloudOff v-else-if="appStatus.saveStatus === 'error'" />
        <IconCloud v-else />
      </button>
    </div>

    <div class="window-controls">
      <button class="window-control" type="button" title="最小化" @click.stop="minimizeWindow">
        <IconSubtract />
      </button>
      <button class="window-control" type="button" title="最大化" @click.stop="toggleMaximizeWindow">
        <IconCheckbox />
      </button>
      <button class="window-control close" type="button" title="关闭" @click.stop="closeWindow">
        <IconClose />
      </button>
    </div>
  </header>
</template>

<style scoped>
.window-titlebar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  height: 36px;
  color: var(--app-titlebar-text);
  background: var(--app-titlebar-bg);
  border-bottom: 1px solid var(--app-border);
  user-select: none;
}

.titlebar-drag-region {
  display: flex;
  min-width: 0;
  align-items: center;
}

.app-brand {
  display: inline-flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
}

.app-logo {
  display: inline-flex;
  width: 18px;
  height: 18px;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  line-height: 1;
  background: #14b8a6;
  border-radius: 5px;
}

.app-title {
  overflow: hidden;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.titlebar-toolbar,
.window-controls {
  display: inline-flex;
  align-items: center;
  height: 100%;
}

.titlebar-toolbar {
  gap: 2px;
  padding-right: 8px;
}

.titlebar-icon-btn,
.window-control {
  display: inline-flex;
  width: 34px;
  height: 30px;
  align-items: center;
  justify-content: center;
  color: inherit;
  cursor: pointer;
  background: transparent;
  border: 0;
  border-radius: 4px;
}

.titlebar-icon-btn:hover,
.window-control:hover {
  background: var(--app-hover);
}

.window-control {
  width: 46px;
  height: 36px;
  border-radius: 0;
}

.window-control.close:hover {
  color: #fff;
  background: #ef4444;
}

.status-saved {
  color: #16a34a;
}

.status-saving {
  color: #1677ff;
}

.status-error {
  color: #ef4444;
}

.spin-icon {
  animation: titlebar-spin 1s linear infinite;
}

.titlebar-macos {
  grid-template-columns: auto minmax(0, 1fr) auto;
}

.titlebar-macos .window-controls {
  grid-column: 1;
  grid-row: 1;
}

.titlebar-macos .titlebar-drag-region {
  grid-column: 2;
  grid-row: 1;
}

.titlebar-macos .titlebar-toolbar {
  grid-column: 3;
  grid-row: 1;
}

@keyframes titlebar-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
