export const supportedLocales = ['zh-CN', 'en'] as const

export type SupportedLocale = (typeof supportedLocales)[number]

const legacyLocaleMap: Record<string, SupportedLocale> = {
  en: 'en',
  'en-us': 'en',
  zh: 'zh-CN',
  'zh-cn': 'zh-CN',
  zhcn: 'zh-CN',
  'zh-tw': 'zh-CN',
  zhtw: 'zh-CN',
}

export function normalizeLocale(value?: string | null): SupportedLocale {
  const raw = (value ?? '').trim()
  if (!raw) return 'zh-CN'
  if (raw === 'zh-CN' || raw === 'en') return raw
  return legacyLocaleMap[raw.toLowerCase()] ?? 'zh-CN'
}
