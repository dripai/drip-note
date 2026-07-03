<script setup lang="ts">
import { computed, onMounted, watchEffect } from 'vue'
import { theme as antTheme } from 'ant-design-vue'
import { usePluginStore } from './stores/pluginStore'
import { useConfigStore } from './stores/configStore'
import WindowTitleBar from './components/WindowTitleBar.vue'

const pluginStore = usePluginStore()
const cfg = useConfigStore()

const isDark = computed(() => cfg.cfg.theme === 'dark')
const antThemeConfig = computed(() => ({
  algorithm: isDark.value ? antTheme.darkAlgorithm : antTheme.defaultAlgorithm,
}))

onMounted(async () => {
  // Load plugins on app start
  // Pass undefined to let service use default backend path
  console.log('[App] Loading plugins...')
  await pluginStore.loadPlugins()
})

watchEffect(() => {
  document.documentElement.dataset.theme = isDark.value ? 'dark' : 'light'
})
</script>

<template>
  <a-config-provider :theme="antThemeConfig">
    <a-app>
      <div class="app" :class="{ 'theme-dark': isDark }">
        <WindowTitleBar />
        <main class="app-content">
          <router-view />
        </main>
      </div>
    </a-app>
  </a-config-provider>
</template>

<style scoped>
.app { display: flex; flex-direction: column; height: 100vh; width: 100%; color: var(--app-text); background: var(--app-bg); }
.app-content { flex: 1; min-height: 0; overflow: hidden; }
</style>

<style>
:root {
  --app-bg: #ffffff;
  --app-panel: #f8fafc;
  --app-surface: #ffffff;
  --app-titlebar-bg: #f8fafc;
  --app-titlebar-text: #111827;
  --app-border: #e5e7eb;
  --app-text: #1f2937;
  --app-muted: #94a3b8;
  --app-hover: #eef2f7;
  --app-active: #e8f3ff;
  --app-active-text: #1677ff;
  --app-split-bg: #ffffff;
  --app-split-line: #e5e7eb;
  --app-split-thumb: #cbd5e1;
}

html[data-theme='dark'] {
  --app-bg: #1e1e1e;
  --app-panel: #181818;
  --app-surface: #1f1f1f;
  --app-titlebar-bg: #181818;
  --app-titlebar-text: #d4d4d4;
  --app-border: #2b2b2b;
  --app-text: #d4d4d4;
  --app-muted: #858585;
  --app-hover: #2a2d2e;
  --app-active: #37373d;
  --app-active-text: #ffffff;
  --app-split-bg: #1e1e1e;
  --app-split-line: #2b2b2b;
  --app-split-thumb: #6a6a6a;
}

html, body, #app { margin: 0; padding: 0; height: 100%; }
body { color: var(--app-text); background: var(--app-bg); }
</style>
