import { ref } from 'vue'

export function useErrorHandler() {
  const lastError = ref<string | null>(null)
  function wrap<T>(fn: () => Promise<T>) {
    return fn().catch(err => { lastError.value = (err?.message ?? String(err)); throw err })
  }
  return { lastError, wrap }
}

