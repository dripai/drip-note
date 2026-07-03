<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConfigStore } from '../../stores/configStore'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import IconBack from '~icons/ri/arrow-left-line'

import GeneralSettings from './General.vue'
import AiSettings from './Ai.vue'
import PluginsSettings from './Plugins.vue'

const cfg = useConfigStore()
const { t } = useI18n()
const router = useRouter()

onMounted(async () => {
  await cfg.init()
  // Ensure AI config is initialized if needed, though AiSettings component also checks this
  if (!cfg.cfg.ai) {
    // We let the store or Ai component handle default creation if missing
    // or we can keep ensureAiConfig here if we want to be safe before rendering
  }
})

type Section = 'general' | 'ai' | 'plugins'
const active = ref<Section>('general')

const nav: { key: Section; labelKey: string; badge?: string }[] = [
  { key: 'general',  labelKey: 'settings_nav.general' },
  { key: 'ai',       labelKey: 'settings_nav.ai' },
  { key: 'plugins',  labelKey: 'settings_nav.plugins' },
]

function goBack() {
  router.back()
}
</script>

<template>
  <div class="settings-view">
    <nav class="settings-nav">
      <div class="nav-list">
        <div
          v-for="item in nav"
          :key="item.key"
          class="nav-item"
          :class="{ active: active === item.key }"
          @click="active = item.key"
        >
          <span>{{ t(item.labelKey) }}</span>
          <span v-if="item.badge" class="badge">{{ item.badge }}</span>
        </div>
      </div>
      
      <div class="back-item" @click="goBack" title="Back">
        <IconBack />
        <span class="back-text">返回</span>
      </div>
    </nav>

    <div class="settings-content">
      <GeneralSettings v-if="active === 'general'" />
      <AiSettings v-else-if="active === 'ai'" />
      <PluginsSettings v-else-if="active === 'plugins'" />
    </div>
  </div>
</template>

<style scoped>
.settings-view { display: flex; height: 100%; width: 100%; flex: 1; min-width: 0; font-size: 13px; }
.settings-nav { width: 140px; border-right: 1px solid #eee; padding: 8px 0; flex-shrink: 0; display: flex; flex-direction: column; }
.nav-list { flex: 1; overflow-y: auto; }
.back-item {
  display: flex; align-items: center; padding: 12px 16px; cursor: pointer; color: #606266; font-size: 14px; font-weight: 500; border-top: 1px solid #f0f0f0; margin-top: auto;
}
.back-item:hover { background: #f5f7fa; color: #409eff; }
.back-text { margin-left: 8px; }
.nav-item { display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; cursor: pointer; font-size: 13px; color: #444; }
.nav-item:hover { background: #f5f5f5; }
.nav-item.active { background: #e8f3ff; color: #1677ff; font-weight: 600; }
.badge { font-size: 10px; color: #aaa; background: #f0f0f0; border-radius: 3px; padding: 1px 4px; }
.settings-content { flex: 1; padding: 24px 32px; overflow-y: auto; width: 100%; min-width: 0; max-width: none; }
.settings-content > section { width: 100%; }
.settings-content :deep(h3),
.settings-content h3 { margin: 0 0 20px; font-size: 15px; font-weight: 600; }
.placeholder-tip { color: #aaa; font-size: 13px; }
</style>
