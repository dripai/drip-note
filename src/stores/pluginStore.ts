import { defineStore } from 'pinia'
import { ref } from 'vue'
import { pluginService } from '../services/plugin/plugin.service'
import type { DripPlugin, PluginManifest } from '../types/plugin'

export const usePluginStore = defineStore('plugin', () => {
  const plugins = ref<PluginManifest[]>([])
  const contentPlugins = ref<DripPlugin[]>([])
  const panelPlugins = ref<DripPlugin[]>([])
  
  async function loadPlugins() {
    await pluginService.loadPlugins()

    // Update local state
    plugins.value = pluginService.getPlugins()
    contentPlugins.value = pluginService.getContentPlugins()
  }

  return {
    plugins,
    contentPlugins,
    panelPlugins,
    loadPlugins
  }
})
