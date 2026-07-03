import { defineStore } from 'pinia'
import type { AppConfig } from '../services/config/config.provider'
import { loadConfig, saveConfig } from '../services/config/config.provider'
import { loadConfigFromSqlite, saveConfigToSqlite } from '../services/config/sqlite.provider'
import { invoke } from '@tauri-apps/api/core'
import { dirname, join } from '@tauri-apps/api/path'
import { createDefaultAiConfig, normalizeAiConfig, type AiConfig } from '../services/ai/ai.types'
import { normalizeLocale } from '../i18n/locale'

export const useConfigStore = defineStore('config', {
  state: () => ({
    cfg: {
      tabsDataDir: '',
      imagesDir: '',
      language: 'zh-CN',
      theme: 'light',
      notesLeftWidth: 160,
      tabsLeftWidth: 260,
      ai: createDefaultAiConfig(),
    } as AppConfig,
  }),
  actions: {
    async init() {
      const fileCfg = await loadConfig()
      const s = await loadConfigFromSqlite()
      this.cfg = s ?? fileCfg
      let updated = false
      const normalizedLanguage = normalizeLocale(this.cfg.language)
      if (this.cfg.language !== normalizedLanguage) {
        this.cfg.language = normalizedLanguage
        updated = true
      }
      if (!this.cfg.ai) this.cfg.ai = createDefaultAiConfig()
      this.cfg.ai = normalizeAiConfig(this.cfg.ai)
      if (this.cfg.theme !== 'dark') this.cfg.theme = 'light'
      if (!this.cfg.imagesDir || !this.cfg.imagesDir.trim()) {
        this.cfg.imagesDir = await this.getDefaultMediaDir('images')
        try { await invoke('fs_ensure_dir', { path: this.cfg.imagesDir }) } catch {}
        updated = true
      }
      if (updated) {
        await saveConfigToSqlite(this.cfg)
      }
      await saveConfig(this.cfg)
    },
    ensureAiConfig(): AiConfig {
      if (!this.cfg.ai) this.cfg.ai = createDefaultAiConfig()
      this.cfg.ai = normalizeAiConfig(this.cfg.ai)
      return this.cfg.ai!
    },
    async setTabsDataDir(dir: string) { this.cfg.tabsDataDir = dir; await this.saveAllConfig() },
    async setLanguage(lang: string) { this.cfg.language = lang; await this.saveAllConfig() },
    async setTheme(theme: 'light' | 'dark') { this.cfg.theme = theme; await this.saveAllConfig() },
    async setImagesDir(dir: string) { this.cfg.imagesDir = dir; await this.saveAllConfig() },
    async setNotesLeftWidth(w: number) { this.cfg.notesLeftWidth = w; await saveConfigToSqlite(this.cfg) },
    async setTabsLeftWidth(w: number) { this.cfg.tabsLeftWidth = w; await saveConfigToSqlite(this.cfg) },
    async saveEditorPrefs() { await saveConfigToSqlite(this.cfg) },
    async saveAllConfig() {
      await saveConfigToSqlite(this.cfg)
      await saveConfig(this.cfg)
    },
    async getDefaultMediaDir(type: 'images'): Promise<string> {
      if (type === 'images' && this.cfg.imagesDir) return this.cfg.imagesDir
      
      const url = await invoke<string>('db_exe_url')
      const p = url.replace(/^sqlite:/, '')
      const dir = await dirname(p)
      return await join(dir, type)
    },
    async saveAiConfig() {
      this.ensureAiConfig()
      await this.saveAllConfig()
    },
  }
})
