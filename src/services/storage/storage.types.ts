export type TreeNode = {
  id: string
  parentId: string | null   // null = root (notebook)
  label: string
  viewType: string           // 'text' | 'markdown'
  content: string
  sortOrder: number
  icon?: string              // emoji icon for root nodes
  color?: string             // color label for root nodes
  tags?: string[]            // free-form text tags
  createdAt: number
  updatedAt: number
  children?: TreeNode[]
}

export interface StorageProvider {
  init(): Promise<void>
  close(): Promise<void>
  tree: {
    listAll(): Promise<TreeNode[]>
    get(id: string): Promise<TreeNode | undefined>
    upsert(node: TreeNode): Promise<void>
    delete(id: string): Promise<void>
    updateSorts(pairs: Array<{ id: string; sortOrder: number }>): Promise<void>
    deleteSubtree(id: string): Promise<void>
  }
  search(query: string): Promise<Array<{ id: string; label: string; snippet: string }>>
}
