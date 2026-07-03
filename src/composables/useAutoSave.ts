import { ref } from 'vue'

export function useAutoSave<T>(handler: (v: T) => Promise<void>, wait = 800) {
  const timer = ref<number | null>(null)
  const pending = ref<T | null>(null)
  let saveChain = Promise.resolve()

  function enqueue(v: T) {
    saveChain = saveChain.catch(() => undefined).then(() => handler(v))
    return saveChain
  }

  function trigger(v: T) {
    if (timer.value) { clearTimeout(timer.value) }
    pending.value = v
    timer.value = window.setTimeout(() => {
      const value = pending.value
      pending.value = null
      timer.value = null
      if (value !== null) void enqueue(value)
    }, wait)
  }

  function flush() {
    if (timer.value) {
      clearTimeout(timer.value)
      timer.value = null
    }
    const value = pending.value
    pending.value = null
    return value !== null ? enqueue(value) : saveChain.catch(() => undefined)
  }

  return { trigger, flush }
}
