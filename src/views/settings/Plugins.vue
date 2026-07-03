<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePluginStore } from '../../stores/pluginStore'
import { storeToRefs } from 'pinia'
import IconDelete from '~icons/ri/delete-bin-line'
import IconDownload from '~icons/ri/download-cloud-line'
import { invoke } from '@tauri-apps/api/core'

interface MarketplacePlugin {
  name: string
  version?: string
  description?: string
  author?: string
  downloadUrl?: string
}

const marketplaceUrl = import.meta.env.VITE_PLUGIN_MARKETPLACE_URL || 'https://dripai.vercel.app/plugins.json'
const { t } = useI18n()
const pluginStore = usePluginStore()
const { plugins: rawPlugins } = storeToRefs(pluginStore)
const plugins = rawPlugins as any
const activeTab = ref('installed')
const marketplacePlugins = ref<MarketplacePlugin[]>([])
const loadingMarketplace = ref(false)
const installingPlugin = ref<string | null>(null)

async function reloadPlugins() {
  await pluginStore.loadPlugins()
}

async function loadMarketplace() {
  loadingMarketplace.value = true
  try {
    const remotePlugins = await invoke<MarketplacePlugin[]>('fetch_marketplace_plugins', { url: marketplaceUrl })
    if (Array.isArray(remotePlugins) && remotePlugins.length > 0) {
      marketplacePlugins.value = remotePlugins
      return
    }
    marketplacePlugins.value = await loadMarketplaceFallback()
  } catch (e) {
    console.error(`Failed to load marketplace from ${marketplaceUrl}:`, e)
    marketplacePlugins.value = await loadMarketplaceFallback()
  } finally {
    loadingMarketplace.value = false
  }
}

async function loadMarketplaceFallback(): Promise<MarketplacePlugin[]> {
  try {
    const res = await fetch('/plugins.json')
    if (!res.ok) return []
    const list = await res.json()
    return Array.isArray(list) ? list : []
  } catch {
    return []
  }
}

async function installPlugin(plugin: any) {
    installingPlugin.value = plugin.name
    try {
        // Assume plugin object has a 'downloadUrl' field
        // If not, construct it or use a default convention
        // For MVP, let's assume we are installing a JS file directly if url ends with .js
        
        // TODO: In real world, downloadUrl should come from marketplace JSON
        const downloadUrl = plugin.downloadUrl || `https://gitee.com/ggtool/drip-note-plugins/raw/master/${plugin.name}/dist/${plugin.name}.umd.js`
        
        await invoke('install_plugin', { url: downloadUrl, name: plugin.name })
        
        // Switch to installed tab and reload
        await reloadPlugins()
        activeTab.value = 'installed'
    } catch (e) {
        console.error('Failed to install plugin:', e)
        alert(`Installation failed: ${e}`)
    } finally {
        installingPlugin.value = null
    }
}

async function togglePlugin(plugin: any) {
    const newStatus = plugin.status === 'enabled' ? 'disabled' : 'enabled'
    try {
        await invoke('update_plugin_status', { id: plugin.id, status: newStatus })
        await reloadPlugins()
    } catch (e) {
        console.error('Failed to update plugin status:', e)
    }
}

async function deletePlugin(plugin: any) {
    if (!confirm(`Are you sure you want to uninstall ${plugin.name}?`)) return
    try {
        await invoke('delete_plugin', { id: plugin.id })
        await reloadPlugins()
    } catch (e) {
        console.error('Failed to delete plugin:', e)
    }
}

onMounted(() => {
  reloadPlugins()
  loadMarketplace()
})
</script>

<template>
  <section class="plugins-settings">
    <div class="settings-header">
      <div class="header-left">
        <h3>{{ t('settings_nav.plugins') }}</h3>
      </div>
    </div>

    <a-tabs v-model:activeKey="activeTab" class="plugin-tabs">
        <a-tab-pane label="Installed" name="installed">
            <div class="plugin-list">
              <a-empty v-if="plugins.length === 0" description="No plugins installed" />
              <a-card v-for="plugin in plugins" :key="plugin.id" class="plugin-card">
                <div class="card-content">
                    <div class="plugin-main">
                        <div class="plugin-header">
                            <span class="plugin-name">{{ plugin.name }}</span>
                            <a-tag size="small" color="default">v{{ plugin.version }}</a-tag>
                            <a-tag size="small" :color="plugin.status === 'enabled' ? 'success' : 'error'">{{ plugin.status }}</a-tag>
                        </div>
                        <p class="plugin-desc">{{ plugin.description || 'No description' }}</p>
                        <p class="plugin-meta" v-if="plugin.author">Author: {{ plugin.author }}</p>
                    </div>
                    <div class="plugin-actions">
                        <a-switch
                            :checked="plugin.status === 'enabled'"
                            @change="togglePlugin(plugin)"
                            checked-children="On"
                            un-checked-children="Off"
                        />
                        <a-button danger type="text" shape="circle" @click="deletePlugin(plugin)" title="Uninstall">
                            <IconDelete />
                        </a-button>
                    </div>
                </div>
              </a-card>
            </div>
        </a-tab-pane>
        
        <a-tab-pane label="Marketplace" name="marketplace">
            <div class="plugin-list" >
                <a-empty v-if="marketplacePlugins.length === 0 && !loadingMarketplace" description="No plugins found in marketplace" />
                <a-card v-for="plugin in marketplacePlugins" :key="plugin.name" class="plugin-card">
                    <div class="card-content">
                        <div class="plugin-main">
                            <div class="plugin-header">
                                <span class="plugin-name">{{ plugin.name }}</span>
                                <a-tag size="small">v{{ plugin.version }}</a-tag>
                            </div>
                            <p class="plugin-desc">{{ plugin.description }}</p>
                            <p class="plugin-meta">Author: {{ plugin.author }}</p>
                        </div>
                        <div class="plugin-actions">
                            <a-button type="primary" size="small" @click="installPlugin(plugin)" :loading="installingPlugin === plugin.name">
                                Install <IconDownload class="ml-1"/>
                            </a-button>
                        </div>
                    </div>
                </a-card>
            </div>
        </a-tab-pane>
    </a-tabs>
  </section>
</template>

<style scoped>
.plugins-settings {
  padding: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.settings-header {
  margin-bottom: 10px;
}

.plugin-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-bottom: 20px;
}

.plugin-card {
    border-radius: 8px;
}

.card-content {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
}

.plugin-main {
    flex: 1;
}

.plugin-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
}

.plugin-name {
    font-weight: 600;
    font-size: 16px;
}

.plugin-desc {
    color: #666;
    margin: 4px 0;
    font-size: 14px;
}

.plugin-meta {
    color: #999;
    font-size: 12px;
    margin: 0;
}

.plugin-actions {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-left: 20px;
}

.ml-1 {
    margin-left: 4px;
}
</style>
