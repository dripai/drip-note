import type { Component } from 'vue'

export interface PluginManifest {
  id: string
  name: string
  version: string
  description?: string
  author?: string
  main: string // Entry point file (e.g., dist/index.js)
  globalName?: string // Global variable name (if UMD)
  permissions?: string[]
}

export interface ContentPlugin {
  id: string
  name: string
  viewType: string
  component: Component
  icon?: string
}

export interface PanelPlugin {
  id: string
  name: string
  panelId: string
  component: Component
  icon: string
  defaultWidth?: number
  defaultPosition?: 'left' | 'right'
}

export type DripPlugin = ContentPlugin | PanelPlugin

export interface Command {
  id: string
  handler: (...args: any[]) => any
  title?: string // For Command Palette
}

export interface DripNoteAPI {
  commands: {
    register(id: string, handler: Function, title?: string): void
    execute(id: string, ...args: any[]): Promise<any>
  }
  notes: {
    getCurrent(): any // Replace with Note type
    getRoots(): any[] // Get all root notebooks
    create(data: any): Promise<any>
    update(id: string, data: any): Promise<void>
  }
  editor?: {
    insertAtCursor(content: string): void
    getSelection(): string
  }
  panel?: {
    emit(event: string, data: any): void
    on(event: string, handler: Function): void
  }
  storage: {
    get(key: string): Promise<any>
    set(key: string, value: any): Promise<void>
  }
  ui: {
    showNotification(message: string): void
    openDialog(component: Component): Promise<any>
  }
}

export interface PluginContext {
  api: DripNoteAPI
}

export interface PluginInstance {
  activate(context: PluginContext): void
  deactivate(): void
  exports?: DripPlugin[]
}
