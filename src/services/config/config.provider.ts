import { readTextFile, writeFile } from '@tauri-apps/plugin-fs'
import { appDataDir, join } from '@tauri-apps/api/path'
import { mkdir } from '@tauri-apps/plugin-fs'
import type { AiConfig } from '../ai/ai.types'
import { createDefaultAiConfig, normalizeAiConfig } from '../ai/ai.types'
import { normalizeLocale } from '../../i18n/locale'

export type AppConfig = {
  tabsDataDir: string
  imagesDir?: string
  language: string
  theme?: 'light' | 'dark'
  editorPrefs?: Record<string, unknown>
  notesLeftWidth?: number
  tabsLeftWidth?: number
  dbFile?: string
  logLevel?: string
  ai?: AiConfig
}

const CONFIG_FILE = 'config.json'

async function getConfigPath() {
  const dir = await appDataDir()
  // Remove 'drip-note' subdirectory to align with backend structure
  // appDataDir() already returns .../AppData/Roaming/com.drip.note
  return await join(dir, CONFIG_FILE)
}

export async function loadConfig(): Promise<AppConfig> {
  try {
    const file = await getConfigPath()
    const txt = await readTextFile(file)
    const parsed = JSON.parse(txt) as AppConfig
    if (parsed.notesLeftWidth === undefined) parsed.notesLeftWidth = 160
    if (parsed.tabsLeftWidth === undefined) parsed.tabsLeftWidth = 260
    if (parsed.dbFile === undefined || parsed.dbFile === null) parsed.dbFile = '.'
    if (!parsed.logLevel) parsed.logLevel = 'debug'
    if (parsed.theme !== 'dark') parsed.theme = 'light'
    parsed.language = normalizeLocale(parsed.language)
    if (!parsed.ai) parsed.ai = createDefaultAiConfig()
    else parsed.ai = normalizeAiConfig(parsed.ai)
    return parsed
  } catch {
    return { tabsDataDir: '', imagesDir: '', language: 'zh-CN', theme: 'light', notesLeftWidth: 160, tabsLeftWidth: 260, dbFile: '.', ai: createDefaultAiConfig() }
  }
}

export async function saveConfig(cfg: AppConfig): Promise<void> {
  const file = await getConfigPath()
  // No need to create subfolder anymore, just ensure appDataDir exists
  const dir = await appDataDir()
  await mkdir(dir, { recursive: true })
  
  cfg.language = normalizeLocale(cfg.language)
  if (!cfg.ai) cfg.ai = createDefaultAiConfig()
  else cfg.ai = normalizeAiConfig(cfg.ai)
  const content = new TextEncoder().encode(JSON.stringify(cfg, null, 2))
  await writeFile(file, content)
}
