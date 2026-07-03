import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import type { DripNoteAPI, PluginManifest, PluginInstance, ContentPlugin, Command } from '../../types/plugin'
import { useTreeStore } from '../../stores/treeStore'

class PluginService {
  private plugins = new Map<string, PluginInstance>()
  private manifests = new Map<string, PluginManifest>()
  private commands = new Map<string, Command>()
  
  // The directory where plugins are stored
  // In production, this might be AppData/plugins
  // For dev, it could be a local path
  private pluginDir: string | null = null

  constructor() {
    // Initialize API implementation
    this.api = this.createAPI()
  }

  private api: DripNoteAPI

  private createAPI(): DripNoteAPI {
    return {
      commands: {
        register: (id: string, handler: Function, title?: string) => {
          if (this.commands.has(id)) {
            console.warn(`[PluginAPI] Command ${id} already exists, overwriting.`)
          }
          // Wrap handler with try-catch
          const safeHandler = async (...args: any[]) => {
            try {
              return await handler(...args)
            } catch (e) {
              console.error(`[PluginAPI] Command ${id} handler failed:`, e)
              throw e
            }
          }
          this.commands.set(id, { id, handler: safeHandler, title })
          this.log('info', `Command registered: ${id}`)
        },
        execute: async (id: string, ...args: any[]) => {
           const cmd = this.commands.get(id)
           if (!cmd) {
             console.warn(`[PluginAPI] Command ${id} not found`)
             return Promise.reject(`Command ${id} not found`)
           }
           return await cmd.handler(...args)
        }
      },
      notes: {
        getCurrent: () => {
          try {
            const store = useTreeStore()
            // Warning: Returns a reactive object. 
            // Plugins can modify this directly, which will trigger UI updates.
            return store.current
          } catch (e) {
            console.error('[PluginAPI] notes.getCurrent failed:', e)
            return null
          }
        },
        getRoots: () => {
          try {
            const store = useTreeStore()
            return store.roots
          } catch (e) {
            console.error('[PluginAPI] notes.getRoots failed:', e)
            return []
          }
        },
        create: async (data) => {
          try {
            const store = useTreeStore()
            if (data.parentId) {
              return await store.addChild(data.parentId, data.label, data.viewType)
            } else {
              return await store.addRoot(data.label, data.viewType)
            }
          } catch (e) {
            console.error('[PluginAPI] notes.create failed:', e)
            throw e
          }
        },
        update: async (id, data) => {
          try {
            const store = useTreeStore()
            if (data.label) await store.saveLabel(id, data.label)
            // TODO: more update fields
          } catch (e) {
            console.error('[PluginAPI] notes.update failed:', e)
            throw e
          }
        }
      },
      editor: {
        insertAtCursor: (content) => {
          console.log('Plugin API: editor.insertAtCursor called', content)
        },
        getSelection: () => {
          return ''
        }
      },
      panel: {
        emit: (event, data) => {
          console.log('Plugin API: panel.emit called', event, data)
        },
        on: (event, _handler) => {
          console.log('Plugin API: panel.on called', event)
        }
      },
      storage: {
        get: async (key) => {
          console.log('Plugin API: storage.get called', key)
          return null
        },
        set: async (key, value) => {
          console.log('Plugin API: storage.set called', key, value)
        }
      },
      ui: {
        showNotification: (message) => {
          console.log('Plugin API: ui.showNotification called', message)
        },
        openDialog: async (_component) => {
          console.log('Plugin API: ui.openDialog called')
        }
      }
    }
  }

  public async init() {
    this.log('info', 'Initializing PluginService...')
    await this.loadPlugins()
  }

  public async loadPlugins() {
    try {
      // 1. Resolve plugin directory from Rust
      this.pluginDir = await invoke<string>('get_plugins_dir')
      this.log('info', `Plugin directory: ${this.pluginDir}`)

      // 2. Scan disk and sync plugin metadata into DB
      const installedPlugins = await invoke<unknown>('scan_and_register_plugins')
      const records = Array.isArray(installedPlugins) ? installedPlugins : []
      if (!Array.isArray(installedPlugins)) {
        console.warn('[PluginService] scan_and_register_plugins returned non-array payload, fallback to empty list.')
      }

      this.log('info', `Scanned and registered ${records.length} plugins in DB`)

      // 3. Load enabled plugins
      await this.loadPluginsFromRecords(records)
    } catch (e) {
      console.error('[PluginService] Load plugins failed:', e)
    }
  }

  // Helper to log to both console and backend
  private async log(level: string, message: string, pluginId?: string) {
      const prefix = pluginId ? `[Plugin:${pluginId}]` : '[PluginService]'
      const fullMessage = `${prefix} ${message}`
      console.log(`[${level}] ${fullMessage}`)
      try {
          await invoke('log_message', { level, message: fullMessage })
      } catch (e) {
          console.error(`Failed to send log to backend: ${e}`)
      }
  }

  // Load plugins from database records
  private async loadPluginsFromRecords(records: any[]) {
    this.plugins.clear()
    this.manifests.clear()
    this.commands.clear()

    for (const record of records) {
      if (record.status !== 'enabled') {
        this.log('info', `Skipping disabled plugin: ${record.name}`)
        continue
      }

      try {
        await this.loadPluginFromRecord(record)
      } catch (e) {
        console.error(`[PluginService] Failed to load plugin ${record.name}:`, e)
      }
    }
  }

  private async loadPluginFromRecord(record: any) {
    const manifest: PluginManifest = {
      name: record.name,
      id: record.id,
      version: record.version,
      description: record.description,
      main: 'dist/index.js', // Assume standard structure or read from record if we stored it
      // Add other fields if needed
    }
    
    // Construct entry path
    // Rust path is absolute, but for webview we might need to convert it
    // If record.path is "C:\Users\...\plugins\demo", we need to load "C:\Users\...\plugins\demo\dist\index.js"
    // But we need to use convertFileSrc to load local files in WebView
    
    // For simplicity, let's assume the standard structure:
    // plugin_root/
    //   package.json
    //   dist/
    //     index.js (or whatever is in 'main')
    
    // We can try to read package.json again here to get 'main', or trust standard convention
    // Let's trust standard convention for now: dist/{name}.umd.js or just read package.json via FS API if needed
    
    // Actually, to be robust, we should read package.json content using FS API
    // But we already have metadata in DB record. Let's assume the main file is at dist/index.js or dist/{name}.umd.js
    // Wait, our CLI generates: dist/{name}.umd.js.
    // So we need to know the exact main file.
    
    // Let's use fs.readTextFile to read package.json from the absolute path to get 'main'
    let mainFile = 'dist/index.js'
    try {
       // record.path is the absolute path to the plugin directory
       // We can't directly use fs.readTextFile with absolute path unless it's in scope?
       // Tauri fs plugin usually requires scope permissions.
       // But we can use our Rust command to read text file if needed, or just standard fetch with convertFileSrc?
       // Let's try convertFileSrc to load package.json
       
       // Actually, we can just guess dist/{name}.umd.js based on our CLI convention
       // Or, we can update the Rust scan logic to store 'main' field in DB.
       // Let's update Rust logic to store 'main' (I missed that in DB schema, but PluginMetadata struct has it)
       // Checking DB schema... I didn't add 'main' column.
       // For now, let's try to load package.json via fetch
       
       const pkgUrl = convertFileSrc(`${record.path}\\package.json`)
       const response = await fetch(pkgUrl)
       if (response.ok) {
         const pkg = await response.json()
         if (pkg.main) mainFile = pkg.main
         manifest.version = pkg.version || manifest.version
         manifest.id = pkg.name // Ensure ID matches name
       }
    } catch (e) {
      console.warn(`[PluginService] Could not read package.json for ${record.name}, trying default main.`)
    }

    const scriptUrl = convertFileSrc(`${record.path}\\${mainFile}`)
    
    this.log('info', `Loading plugin ${manifest.name} from ${scriptUrl}`)
    
    await this.loadPluginScript(scriptUrl, manifest)
  }

  // Reuse existing loadPluginScript logic, but adapted
  private async loadPluginScript(url: string, manifest: PluginManifest) {
      // Create script tag
      return new Promise<void>((resolve, reject) => {
      const script = document.createElement('script')
      script.src = url
      script.type = 'module' // or 'text/javascript' for UMD? UMD is safer usually
      // For UMD, we don't use type=module.
      // But wait, our build produces UMD.
      
      script.onload = async () => {
        // ... (existing logic to find global variable)
        // We need to know the global variable name.
        // Convention: PascalCase of pkg.name
        // e.g. my-picture -> MyPicture
        const globalName = this.toPascalCase(manifest.name)
        // @ts-ignore
        const pluginModule = window[globalName]
        
        if (!pluginModule) {
           // Try to find it in window keys just in case?
           console.error(`[Plugin: ${manifest.name}] Global variable '${globalName}' not found.`)
           // Fallback: check if it registered itself in a different way?
           // For now, fail.
           reject(`Global variable '${globalName}' not found`)
           return
        }
        
        try {
            const context = { api: this.api, manifest }
            const instance = pluginModule.activate(context)
            if (instance) {
                this.plugins.set(manifest.name, instance)
                this.manifests.set(manifest.name, manifest)
                this.log('info', `[Plugin: ${manifest.name}] Activated successfully`)
                resolve()
            } else {
                reject('activate() returned null')
            }
        } catch (e) {
            reject(e)
        }
      }
      
      script.onerror = (e) => reject(e)
      document.head.appendChild(script)
    })
  }

  private toPascalCase(str: string) {
    return str
      .replace(/(?:^|-)(\w)/g, (_, c) => c.toUpperCase())
      .replace(/[^a-zA-Z0-9]/g, '')
  }

  getPlugins() {
    return Array.from(this.manifests.values())
  }

  getContentPlugins(): ContentPlugin[] {
    const list: ContentPlugin[] = []
    for (const [id, module] of this.plugins) {
      const exports = (module as any)._exports
      if (exports && exports.viewType && exports.component) {
        // Use directory name as plugin ID, but keep viewType from exports
        list.push({
          id: id, 
          name: exports.name || id,
          viewType: exports.viewType,
          component: exports.component,
          icon: exports.icon
        })
      }
    }
    return list
  }
}

export const pluginService = new PluginService()
