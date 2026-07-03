import { defineStore } from 'pinia'
import { sqliteProvider } from '../services/storage/sqlite.provider'
import type { TreeNode } from '../services/storage/storage.types'


function buildTree(flat: TreeNode[]): TreeNode[] {
  const map = new Map<string, TreeNode>()
  const roots: TreeNode[] = []
  for (const n of flat) { map.set(n.id, { ...n, children: [] }) }
  for (const n of map.values()) {
    if (n.parentId === null) roots.push(n)
    else map.get(n.parentId!)?.children?.push(n)
  }
  const sort = (nodes: TreeNode[]) => {
    nodes.sort((a, b) => a.sortOrder - b.sortOrder)
    nodes.forEach(n => n.children && sort(n.children))
  }
  sort(roots)
  return roots
}

export const useTreeStore = defineStore('tree', {
  state: () => ({
    flat: [] as TreeNode[],       // all nodes flat list
    roots: [] as TreeNode[],      // built tree (roots with nested children)
    currentId: null as string | null,
    current: null as TreeNode | null,
  }),

  actions: {
    async init() {
      this.flat = await sqliteProvider.tree.listAll()
      this.roots = buildTree(this.flat)
    },

    async selectNode(id: string) {
      const node = this.flat.find(n => n.id === id) ?? await sqliteProvider.tree.get(id)
      if (!node) return
      this.currentId = id
      this.current = { ...node }
    },

    async saveContent(content: string) {
      if (!this.current) return
      await this.saveNodeContent(this.current.id, content)
    },

    async saveNodeContent(id: string, content: string) {
      const node = this.flat.find(n => n.id === id) ?? await sqliteProvider.tree.get(id)
      if (!node) return
      node.content = content
      node.updatedAt = Date.now()
      await sqliteProvider.tree.upsert(node)
      const idx = this.flat.findIndex(n => n.id === id)
      if (idx >= 0) this.flat[idx] = { ...node }
      if (this.current?.id === id) {
        this.current = { ...node }
      }
    },

    async saveNodeProperties(id: string, icon: string | undefined, tags: string[] | undefined) {
      const node = this.flat.find(n => n.id === id)
      if (!node) return
      node.icon = icon
      node.tags = tags
      node.updatedAt = Date.now()
      await sqliteProvider.tree.upsert(node)
      await this.init()
    },

    async saveLabel(id: string, label: string) {
      const node = this.flat.find(n => n.id === id)
      if (!node) return
      node.label = label
      node.updatedAt = Date.now()
      await sqliteProvider.tree.upsert(node)
      await this.init()
    },

    async addRoot(label: string, viewType = 'text') {
      const now = Date.now()
      const siblings = this.flat.filter(n => n.parentId === null)
      const maxSort = siblings.reduce((m, n) => Math.max(m, n.sortOrder), 0)
      const node: TreeNode = { id: crypto.randomUUID(), parentId: null, label, viewType, content: '', sortOrder: maxSort + 1, createdAt: now, updatedAt: now }
      await sqliteProvider.tree.upsert(node)
      await this.init()
      return node.id
    },

    async addChild(parentId: string, label: string, viewType = 'text') {
      const now = Date.now()
      const siblings = this.flat.filter(n => n.parentId === parentId)
      const maxSort = siblings.reduce((m, n) => Math.max(m, n.sortOrder), 0)
      const node: TreeNode = { id: crypto.randomUUID(), parentId, label, viewType, content: '', sortOrder: maxSort + 1, createdAt: now, updatedAt: now }
      await sqliteProvider.tree.upsert(node)
      await this.init()
      return node.id
    },

    async deleteNode(id: string): Promise<boolean> {
      const hasChildren = this.flat.some(n => n.parentId === id)
      if (hasChildren) return false
      await sqliteProvider.tree.delete(id)
      if (this.currentId === id) { this.currentId = null; this.current = null }
      await this.init()
      return true
    },

    async moveNode(id: string, newParentId: string | null) {
      const node = this.flat.find(n => n.id === id)
      if (!node) return
      const siblings = this.flat.filter(n => n.parentId === newParentId && n.id !== id)
      const maxSort = siblings.reduce((m, n) => Math.max(m, n.sortOrder), 0)
      node.parentId = newParentId
      node.sortOrder = maxSort + 1
      node.updatedAt = Date.now()
      await sqliteProvider.tree.upsert(node)
      await this.init()
    },

    async reorderSiblings(_parentId: string | null, orderedIds: string[]) {
      const pairs = orderedIds.map((id, i) => ({ id, sortOrder: i + 1 }))
      await sqliteProvider.tree.updateSorts(pairs)
      await this.init()
    },

  },
})
