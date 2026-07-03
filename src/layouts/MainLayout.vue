<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useConfigStore } from '../stores/configStore'
import { useI18n } from 'vue-i18n'
import { normalizeLocale } from '../i18n/locale'

import { invoke } from '@tauri-apps/api/core'

const cfg = useConfigStore()
const { locale } = useI18n()

onMounted(async () => {
  await cfg.init()
  locale.value = normalizeLocale(cfg.cfg.language)
  // Apply log level from config
  if (cfg.cfg.logLevel) {
    invoke('update_log_level', { level: cfg.cfg.logLevel }).catch(console.error)
  }
})

watch(() => cfg.cfg.language, (l) => { locale.value = normalizeLocale(l) })

</script>

<template>
  <div class="layout-app">
    <router-view />
  </div>
</template>

<style scoped>
.layout-app { display: flex; height: 100%; overflow: hidden; color: var(--app-text); background: var(--app-bg); }
</style>
