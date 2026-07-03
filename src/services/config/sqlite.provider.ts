import Database from '@tauri-apps/plugin-sql'
import { getDbUrl, setDbFile } from '../storage/sqlite.provider'
import type { AppConfig } from './config.provider'
import { createDefaultAiConfig, normalizeAiConfig } from '../ai/ai.types'
import { normalizeLocale } from '../../i18n/locale'

async function getDb() {
  if (getDbUrl() === 'sqlite:drip-note.db') await setDbFile('')
  const url = getDbUrl()
  return await Database.load(url)
}

export async function loadConfigFromSqlite(): Promise<AppConfig | null> {
  try {
    const db = await getDb()
    const rows = await db.select<Array<{ key: string; value: string }>>('SELECT key, value FROM app_config')
    const map = new Map(rows.map(r => [r.key, r.value]))
    const cfg: AppConfig = {
      tabsDataDir: map.get('tabs_data_dir') ?? '',
      imagesDir: map.get('images_dir') ?? '',
      language: normalizeLocale(map.get('language') ?? 'zh-CN'),
      theme: (map.get('theme') as any) ?? undefined,
      editorPrefs: map.get('editor_prefs') ? JSON.parse(map.get('editor_prefs')!) : undefined,
      notesLeftWidth: map.get('notes_left_width') ? Number(map.get('notes_left_width')) : 160,
      tabsLeftWidth: map.get('tabs_left_width') ? Number(map.get('tabs_left_width')) : 260,
      dbFile: map.get('db_file') ?? '.',
      logLevel: map.get('log_level') ?? 'debug',
      ai: map.get('ai_config') ? normalizeAiConfig(JSON.parse(map.get('ai_config')!)) : createDefaultAiConfig(),
    }
    return cfg
  } catch {
    return null
  }
}

export async function saveConfigToSqlite(cfg: AppConfig): Promise<void> {
  const db = await getDb()
  const now = Date.now()
  const entries: Array<[string, string]> = [
    ['tabs_data_dir', cfg.tabsDataDir],
    ['images_dir', cfg.imagesDir ?? ''],
    ['language', normalizeLocale(cfg.language)],
    ['theme', (cfg.theme ?? '') as any],
    ['editor_prefs', JSON.stringify(cfg.editorPrefs ?? {})],
    ['notes_left_width', String(cfg.notesLeftWidth ?? 160)],
    ['tabs_left_width', String(cfg.tabsLeftWidth ?? 260)],
    ['db_file', (cfg.dbFile ?? '.')],
    ['log_level', (cfg.logLevel ?? 'debug')],
    ['ai_config', JSON.stringify(cfg.ai ?? createDefaultAiConfig())],
  ]
  for (const [k, v] of entries) {
    await db.execute('INSERT OR REPLACE INTO app_config (key, value, updated_at) VALUES ($1, $2, $3)', [k, v, now])
  }
}
