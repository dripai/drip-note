<script setup lang="ts">
import { ref, onMounted, shallowRef, watch } from 'vue'

const props = defineProps<{
  pluginPath: string
  context?: any
}>()

const Component = shallowRef<any>(null)
const error = ref<string | null>(null)
const loading = ref(false)

async function loadPlugin() {
  if (!props.pluginPath) return
  
  loading.value = true
  error.value = null
  
  try {
    // 1. Load the UMD script
    // In Tauri, we might need to convert local path to asset URL or read file content
    // For now assuming we can load via script tag (might need convertFileSrc)
    // Or we read file content and eval it (less secure but works for local files)
    
    // For MVP, we assume pluginPath is a valid URL or asset path
    // In production, we need a robust loader that handles `convertFileSrc`
    
    // Mock implementation:
    // await loadScript(props.pluginPath)
    // const plugin = (window as any).DripPlugins[pluginName]
    // Component.value = markRaw(plugin.default || plugin)
    
    // Temporary placeholder
    Component.value = null
    error.value = "Plugin loading not fully implemented yet"
  } catch (e: any) {
    error.value = `Failed to load plugin: ${e.message}`
    console.error(e)
  } finally {
    loading.value = false
  }
}

watch(() => props.pluginPath, loadPlugin)
onMounted(loadPlugin)
</script>

<template>
  <div class="plugin-container">
    <div v-if="loading" class="loading">Loading plugin...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <component 
      v-else-if="Component" 
      :is="Component" 
      v-bind="context" 
    />
  </div>
</template>

<style scoped>
.plugin-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
}
.loading, .error {
  padding: 20px;
  text-align: center;
  color: #888;
}
.error {
  color: #ff4d4f;
}
</style>