<script setup lang="ts">
import { computed, ref } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { useConfigStore } from '../../stores/configStore'
import type { AiModel } from '@/services/ai/ai.types'

const cfg = useConfigStore()

const aiConfig = computed(() => cfg.ensureAiConfig())
const models = computed(() => aiConfig.value.models)
const dialogOpen = ref(false)
const editing = ref<AiModel | null>(null)

function createEmptyModel(): AiModel {
  return {
    id: crypto.randomUUID(),
    name: '',
    baseUrl: '',
    apiKey: '',
    model: '',
    enabled: true,
  }
}

function openModelDialog(model?: AiModel) {
  editing.value = model ? { ...model } : createEmptyModel()
  dialogOpen.value = true
}

async function saveModel() {
  if (!editing.value) return
  const model = editing.value
  if (!model.name.trim() || !model.baseUrl.trim() || !model.model.trim()) {
    message.error('Name, API URL, and model are required')
    return
  }

  const config = cfg.ensureAiConfig()
  const index = config.models.findIndex((item) => item.id === model.id)
  if (index >= 0) config.models[index] = { ...model }
  else config.models.push({ ...model })
  if (!config.defaultModelId) config.defaultModelId = model.id
  config.lastUpdated = Date.now()
  await cfg.saveAiConfig()
  dialogOpen.value = false
  message.success('AI model saved')
}

async function toggleModel(model: AiModel, enabled: boolean) {
  const config = cfg.ensureAiConfig()
  const target = config.models.find((item) => item.id === model.id)
  if (!target) return
  target.enabled = enabled
  config.lastUpdated = Date.now()
  await cfg.saveAiConfig()
}

async function setDefaultModel(model: AiModel) {
  const config = cfg.ensureAiConfig()
  config.defaultModelId = model.id
  config.lastUpdated = Date.now()
  await cfg.saveAiConfig()
  message.success('Default AI model updated')
}

function removeModel(model: AiModel) {
  Modal.confirm({
    title: 'Delete AI model',
    content: `Delete "${model.name || model.model}"?`,
    okText: 'Delete',
    cancelText: 'Cancel',
    okType: 'danger',
    async onOk() {
      const config = cfg.ensureAiConfig()
      config.models = config.models.filter((item) => item.id !== model.id)
      if (config.defaultModelId === model.id) config.defaultModelId = config.models[0]?.id
      config.lastUpdated = Date.now()
      await cfg.saveAiConfig()
      message.success('AI model deleted')
    },
  })
}
</script>

<template>
  <section class="ai-settings">
    <div class="section-header">
      <div>
        <h3>AI</h3>
        <p class="section-desc">Configure one or more OpenAI-compatible API endpoints.</p>
      </div>
      <a-button type="primary" @click="openModelDialog()">Add Model</a-button>
    </div>

    <a-table :data-source="models" row-key="id" size="small" bordered :pagination="false">
      <a-table-column title="Default" width="90">
        <template #default="{ record }">
          <a-radio
            :checked="aiConfig.defaultModelId === record.id"
            @change="setDefaultModel(record)"
          />
        </template>
      </a-table-column>
      <a-table-column title="Name" data-index="name" min-width="160" />
      <a-table-column title="API URL" data-index="baseUrl" min-width="260" ellipsis />
      <a-table-column title="Model" data-index="model" min-width="180" />
      <a-table-column title="Enabled" width="100">
        <template #default="{ record }">
          <a-switch
            :checked="record.enabled"
            size="small"
            @change="(checked: boolean) => toggleModel(record, checked)"
          />
        </template>
      </a-table-column>
      <a-table-column title="Actions" width="150">
        <template #default="{ record }">
          <a-button type="link" size="small" @click="openModelDialog(record)">Edit</a-button>
          <a-button type="link" danger size="small" @click="removeModel(record)">Delete</a-button>
        </template>
      </a-table-column>
    </a-table>

    <a-modal
      v-model:open="dialogOpen"
      title="AI Model"
      width="560px"
      :destroy-on-close="true"
      @ok="saveModel"
    >
      <a-form v-if="editing" :label-col="{ style: { width: '96px' } }">
        <a-form-item label="Name" required>
          <a-input v-model:value="editing.name" placeholder="My API" />
        </a-form-item>
        <a-form-item label="API URL" required>
          <a-input v-model:value="editing.baseUrl" placeholder="https://api.example.com/v1" />
        </a-form-item>
        <a-form-item label="API Key">
          <a-input-password v-model:value="editing.apiKey" />
        </a-form-item>
        <a-form-item label="Model" required>
          <a-input v-model:value="editing.model" placeholder="gpt-4.1-mini" />
        </a-form-item>
        <a-form-item label="Enabled">
          <a-switch v-model:checked="editing.enabled" />
        </a-form-item>
      </a-form>
    </a-modal>
  </section>
</template>

<style scoped>
.ai-settings { width: 100%; }
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 16px;
}
.section-header h3 {
  margin-bottom: 4px;
}
.section-desc {
  margin: 0;
  color: #777;
  font-size: 13px;
}
</style>
