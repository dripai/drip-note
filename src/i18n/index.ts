import { createI18n } from 'vue-i18n'
import en from './locales/en'
import zhCN from './locales/zh-cn'

const messages = {
  'en': en,
  'zh-CN': zhCN,
}

export const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'en',
  messages,
})
