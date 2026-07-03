import { defineStore } from 'pinia'

export type SaveStatus = 'idle' | 'saving' | 'saved' | 'error'

export const useAppStatusStore = defineStore('appStatus', {
  state: () => ({
    saveStatus: 'idle' as SaveStatus,
    lastSavedAt: 0,
    saveRequestId: 0,
  }),
  actions: {
    setSaving() {
      this.saveStatus = 'saving'
    },
    setSaved() {
      this.saveStatus = 'saved'
      this.lastSavedAt = Date.now()
    },
    setError() {
      this.saveStatus = 'error'
    },
    requestSave() {
      this.saveRequestId += 1
    },
  },
})
