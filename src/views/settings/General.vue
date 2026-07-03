<script setup lang="ts">
import { computed, reactive, watch } from 'vue'
import { useConfigStore } from '../../stores/configStore'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { message } from 'ant-design-vue'
import { normalizeLocale } from '../../i18n/locale'

const cfg = useConfigStore()
const { t, locale } = useI18n()

const form = reactive({
  language: normalizeLocale(cfg.cfg.language),
  logLevel: cfg.cfg.logLevel || 'debug',
})

watch(
  () => cfg.cfg.language,
  (val) => {
    form.language = normalizeLocale(val)
  },
  { immediate: true },
)

watch(
  () => cfg.cfg.logLevel,
  (val) => {
    form.logLevel = val || 'debug'
  },
  { immediate: true },
)

watch(
  () => form.language,
  (val) => {
    locale.value = normalizeLocale(val)
  },
)

const hasChanges = computed(() => {
  return normalizeLocale(form.language) !== normalizeLocale(cfg.cfg.language) || (form.logLevel || 'debug') !== (cfg.cfg.logLevel || 'debug')
})

async function saveGeneralSettings() {
  try {
    cfg.cfg.language = normalizeLocale(form.language)
    cfg.cfg.logLevel = form.logLevel || 'debug'
    locale.value = cfg.cfg.language
    await cfg.saveAllConfig()
    await invoke('update_log_level', { level: cfg.cfg.logLevel })
    message.success(t('settings.save_success'))
  } catch (e) {
    console.error('Failed to save general settings:', e)
    message.error(t('settings.save_failed'))
  }
}
</script>

<template>
  <section class="general-settings">
    <h3>{{ t('settings_nav.general') }}</h3>
    <a-form :label-col="{ style: { width: '120px' } }" size="small">
      <a-form-item :label="t('settings.language')">
        <a-select v-model:value="form.language" style="width:200px">
          <a-select-option value="en">English</a-select-option>
          <a-select-option value="zh-CN">????</a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item :label="t('settings.log_level')">
        <a-select v-model:value="form.logLevel" style="width:200px">
          <a-select-option value="debug">Debug</a-select-option>
          <a-select-option value="info">Info</a-select-option>
          <a-select-option value="warn">Warn</a-select-option>
          <a-select-option value="error">Error</a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item>
        <a-button type="primary" :disabled="!hasChanges" @click="saveGeneralSettings">{{ t('common.save') }}</a-button>
      </a-form-item>
    </a-form>
  </section>
</template>
