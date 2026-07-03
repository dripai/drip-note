import { readTextFile, writeFile } from '@tauri-apps/plugin-fs'
import { join } from '@tauri-apps/api/path'

export type TabItem = { id: string; title: string; url: string; folderId?: string; createdAt: number; updatedAt: number }
export type TabFolder = { id: string; name: string; sortOrder?: number }
export type TabData = { items: TabItem[]; folders: TabFolder[] }

const FILE_NAME = 'tabs.json'

export async function loadTabs(dataDir: string): Promise<TabData> {
  try {
    const file = await join(dataDir, FILE_NAME)
    const txt = await readTextFile(file)
    return JSON.parse(txt) as TabData
  } catch {
    return { items: [], folders: [] }
  }
}

export async function saveTabs(dataDir: string, data: TabData): Promise<void> {
  const file = await join(dataDir, FILE_NAME)
  const content = new TextEncoder().encode(JSON.stringify(data, null, 2))
  await writeFile(file, content)
}

export async function importFromBrowserJson(jsonPath: string, existing: TabData): Promise<TabData> {
  const txt = await readTextFile(jsonPath)
  const parsed = JSON.parse(txt)
  // 简化解析：将书签平铺为 TabItem（Chrome/Edge 书签 JSON 有层级，真实实现需递归）
  const now = Date.now()
  const imported: TabItem[] = []
  function walk(node: any, folder?: string) {
    if (node.type === 'url') {
      imported.push({ id: crypto.randomUUID(), title: node.name, url: node.url, folderId: folder, createdAt: now, updatedAt: now })
    } else if (node.children) {
      const fid = folder ?? crypto.randomUUID()
      node.children.forEach((c: any) => walk(c, fid))
    }
  }
  if (parsed.roots) {
    Object.values(parsed.roots).forEach((r: any) => walk(r))
  }
  // 合并策略：简单追加（避免重复匹配复杂度）
  return { items: [...existing.items, ...imported], folders: existing.folders }
}
