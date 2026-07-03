import Database from '@tauri-apps/plugin-sql'
import { invoke } from '@tauri-apps/api/core'
import type { StorageProvider, TreeNode } from './storage.types'

let dbUrl = 'sqlite:drip-note.db'
let schemaEnsured = false
type SqliteConnection = {
  select<T>(query: string, bindValues?: unknown[]): Promise<T>
  execute(query: string, bindValues?: unknown[]): Promise<unknown>
}

async function hasTable(d: SqliteConnection, table: string): Promise<boolean> {
  const rows = await d.select<Array<{ name: string }>>(
    'SELECT name FROM sqlite_master WHERE type = $1 AND name = $2',
    ['table', table],
  )
  return rows.length > 0
}

async function hasColumn(d: SqliteConnection, table: string, column: string): Promise<boolean> {
  const rows = await d.select<Array<{ name: string }>>(`PRAGMA table_info(${table})`)
  return rows.some(row => row.name === column)
}

async function migrateLegacyNotes(d: SqliteConnection) {
  const countRows = await d.select<Array<{ total: number }>>('SELECT COUNT(*) AS total FROM tree_nodes')
  if ((countRows[0]?.total ?? 0) > 0) return

  const hasNotebooks = await hasTable(d, 'notebooks')
  const hasNotes = await hasTable(d, 'notes')
  if (!hasNotebooks && !hasNotes) return

  const now = Date.now()
  const notebookIds = new Set<string>()

  if (hasNotebooks) {
    const notebooks = await d.select<Array<{ id: string; name: string; icon: string | null; sort_order: number | null }>>(
      'SELECT id, name, icon, sort_order FROM notebooks ORDER BY COALESCE(sort_order, 0), name',
    )
    for (const notebook of notebooks) {
      notebookIds.add(notebook.id)
      await d.execute(
        'INSERT OR IGNORE INTO tree_nodes(id, parent_id, label, view_type, content, sort_order, icon, color, tags, created_at, updated_at) VALUES($1, NULL, $2, $3, $4, $5, $6, NULL, NULL, $7, $8)',
        [notebook.id, notebook.name, 'text', '', notebook.sort_order ?? 0, notebook.icon ?? null, now, now],
      )
    }
  }

  if (hasNotes) {
    const hasSort = await hasColumn(d, 'notes', 'sort')
    const hasDeletedAt = await hasColumn(d, 'notes', 'deleted_at')
    const hasTagTables = await hasTable(d, 'note_tags') && await hasTable(d, 'tags')
    const sortExpr = hasSort ? 'COALESCE(n.sort, 0)' : '0'
    const tagSelect = hasTagTables ? 'group_concat(t.name)' : 'NULL'
    const tagJoin = hasTagTables ? 'LEFT JOIN note_tags nt ON nt.note_id = n.id LEFT JOIN tags t ON t.id = nt.tag_id' : ''
    const where = hasDeletedAt ? 'WHERE n.deleted_at IS NULL' : ''
    const groupBy = hasTagTables ? 'GROUP BY n.id' : ''
    const notes = await d.select<Array<{
      id: string
      title: string
      content: string | null
      notebook_id: string | null
      sort_order: number
      created_at: number
      updated_at: number
      tags: string | null
    }>>(
      `SELECT n.id, n.title, COALESCE(n.content_md, '') AS content, n.notebook_id, ${sortExpr} AS sort_order, n.created_at, n.updated_at, ${tagSelect} AS tags FROM notes n ${tagJoin} ${where} ${groupBy} ORDER BY sort_order, n.created_at`,
    )

    for (const note of notes) {
      const parentId = note.notebook_id && notebookIds.has(note.notebook_id) ? note.notebook_id : null
      await d.execute(
        'INSERT OR IGNORE INTO tree_nodes(id, parent_id, label, view_type, content, sort_order, icon, color, tags, created_at, updated_at) VALUES($1, $2, $3, $4, $5, $6, NULL, NULL, $7, $8, $9)',
        [note.id, parentId, note.title, 'markdown', note.content ?? '', note.sort_order ?? 0, note.tags ?? null, note.created_at, note.updated_at],
      )
    }
  }
}

export async function setDbFile(_file: string) {
  dbUrl = await invoke<string>('db_exe_url')
  schemaEnsured = false
}

export function getDbUrl(): string { return dbUrl }

async function ensureSchema(d: SqliteConnection) {
  if (schemaEnsured) return
  await d.execute("CREATE TABLE IF NOT EXISTS app_config (\n  key TEXT PRIMARY KEY,\n  value TEXT NOT NULL,\n  updated_at INTEGER NOT NULL\n);")
  await d.execute("CREATE TABLE IF NOT EXISTS tree_nodes(\n  id TEXT PRIMARY KEY,\n  parent_id TEXT,\n  label TEXT NOT NULL,\n  view_type TEXT NOT NULL DEFAULT 'text',\n  content TEXT NOT NULL DEFAULT '',\n  sort_order INTEGER NOT NULL DEFAULT 0,\n  icon TEXT,\n  color TEXT,\n  tags TEXT,\n  created_at INTEGER NOT NULL,\n  updated_at INTEGER NOT NULL\n);")
  // migrate: add icon/color/tags if missing
  try { await d.execute('ALTER TABLE tree_nodes ADD COLUMN icon TEXT') } catch {}
  try { await d.execute('ALTER TABLE tree_nodes ADD COLUMN color TEXT') } catch {}
  try { await d.execute('ALTER TABLE tree_nodes ADD COLUMN tags TEXT') } catch {}
  await d.execute("CREATE INDEX IF NOT EXISTS idx_tree_nodes_parent ON tree_nodes(parent_id);")
  await d.execute("CREATE VIRTUAL TABLE IF NOT EXISTS tree_nodes_fts USING fts5(id UNINDEXED, label, content, content='tree_nodes', content_rowid='rowid');")
  await d.execute(`CREATE TRIGGER IF NOT EXISTS tree_nodes_fts_insert AFTER INSERT ON tree_nodes BEGIN INSERT INTO tree_nodes_fts(rowid, id, label, content) VALUES (new.rowid, new.id, new.label, new.content); END;`)
  await d.execute(`CREATE TRIGGER IF NOT EXISTS tree_nodes_fts_update AFTER UPDATE ON tree_nodes BEGIN INSERT INTO tree_nodes_fts(tree_nodes_fts, rowid, id, label, content) VALUES ('delete', old.rowid, old.id, old.label, old.content); INSERT INTO tree_nodes_fts(rowid, id, label, content) VALUES (new.rowid, new.id, new.label, new.content); END;`)
  await d.execute(`CREATE TRIGGER IF NOT EXISTS tree_nodes_fts_delete AFTER DELETE ON tree_nodes BEGIN INSERT INTO tree_nodes_fts(tree_nodes_fts, rowid, id, label, content) VALUES ('delete', old.rowid, old.id, old.label, old.content); END;`)
  await migrateLegacyNotes(d)
  schemaEnsured = true
}

async function db() {
  if (!dbUrl || dbUrl === 'sqlite:drip-note.db') {
    dbUrl = await invoke<string>('db_exe_url')
  }
  const d = await Database.load(dbUrl)
  await ensureSchema(d)
  return d
}

export async function testDbConnection(_file: string | null | undefined, allowInit: boolean): Promise<boolean> {
  try {
    const url = await invoke<string>('db_exe_url')
    const d = await Database.load(url)
    if (allowInit) { await ensureSchema(d) } else { await d.select('SELECT 1') }
    await d.close()
    return true
  } catch { return false }
}

function rowToNode(r: any): TreeNode {
  return { id: r.id, parentId: r.parent_id ?? null, label: r.label, viewType: r.view_type ?? 'text', content: r.content ?? '', sortOrder: r.sort_order ?? 0, icon: r.icon ?? undefined, color: r.color ?? undefined, tags: r.tags ? r.tags.split(',').filter(Boolean) : undefined, createdAt: r.created_at, updatedAt: r.updated_at }
}

export const sqliteProvider: StorageProvider = {
  async init() {},
  async close() { const d = await db(); await d.close() },

  tree: {
    async listAll(): Promise<TreeNode[]> {
      const d = await db()
      const rows = await d.select<Array<any>>('SELECT id, parent_id, label, view_type, content, sort_order, icon, color, tags, created_at, updated_at FROM tree_nodes ORDER BY sort_order ASC, created_at ASC')
      return rows.map(rowToNode)
    },
    async get(id: string): Promise<TreeNode | undefined> {
      const d = await db()
      const rows = await d.select<Array<any>>('SELECT id, parent_id, label, view_type, content, sort_order, icon, color, tags, created_at, updated_at FROM tree_nodes WHERE id = $1', [id])
      return rows[0] ? rowToNode(rows[0]) : undefined
    },
    async upsert(node: TreeNode): Promise<void> {
      const d = await db()
      await d.execute(
        'INSERT INTO tree_nodes(id, parent_id, label, view_type, content, sort_order, icon, color, tags, created_at, updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) ON CONFLICT(id) DO UPDATE SET parent_id=$2, label=$3, view_type=$4, content=$5, sort_order=$6, icon=$7, color=$8, tags=$9, updated_at=$11',
        [node.id, node.parentId ?? null, node.label, node.viewType, node.content, node.sortOrder, node.icon ?? null, node.color ?? null, node.tags?.join(',') ?? null, node.createdAt, node.updatedAt]
      )
    },
    async delete(id: string): Promise<void> {
      const d = await db()
      await d.execute('DELETE FROM tree_nodes WHERE id = $1', [id])
    },
    async updateSorts(pairs: Array<{ id: string; sortOrder: number }>): Promise<void> {
      const d = await db()
      for (const p of pairs) await d.execute('UPDATE tree_nodes SET sort_order = $1 WHERE id = $2', [p.sortOrder, p.id])
    },
    async deleteSubtree(id: string): Promise<void> {
      const d = await db()
      const toDelete: string[] = [id]
      let i = 0
      while (i < toDelete.length) {
        const pid = toDelete[i++]
        const children = await d.select<Array<any>>('SELECT id FROM tree_nodes WHERE parent_id = $1', [pid])
        for (const c of children) toDelete.push(c.id)
      }
      for (const nid of toDelete) {
        await d.execute('DELETE FROM tree_nodes WHERE id = $1', [nid])
      }
    },
  },

  async search(query: string): Promise<Array<{ id: string; label: string; snippet: string }>> {
    const d = await db()
    const rows = await d.select<Array<any>>(
      `SELECT id, label, snippet(tree_nodes_fts, 2, '<b>', '</b>', '…', 20) AS snippet FROM tree_nodes_fts WHERE tree_nodes_fts MATCH $1 ORDER BY rank LIMIT 50`,
      [query]
    )
    return rows.map(r => ({ id: r.id, label: r.label, snippet: r.snippet }))
  },
}
