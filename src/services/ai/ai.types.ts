export type AiModel = {
  id: string
  name: string
  baseUrl: string
  apiKey: string
  model: string
  enabled: boolean
}

export type AiConfig = {
  models: AiModel[]
  defaultModelId?: string
  lastUpdated?: number
}

export const DEFAULT_MODELS: AiModel[] = [
  {
    id: 'openai_compatible',
    name: 'OpenAI Compatible',
    baseUrl: 'https://api.openai.com/v1',
    apiKey: '',
    model: 'gpt-4.1-mini',
    enabled: false,
  },
]

export function createDefaultAiConfig(): AiConfig {
  return {
    models: DEFAULT_MODELS.map((model) => ({ ...model })),
    defaultModelId: DEFAULT_MODELS[0]?.id,
    lastUpdated: Date.now(),
  }
}

export function normalizeAiConfig(raw?: Partial<AiConfig> | null): AiConfig {
  const defaults = createDefaultAiConfig()
  if (!raw) return defaults

  const rawModels = Array.isArray(raw.models) ? raw.models : []
  const models = rawModels
    .filter((model: any) => model && typeof model === 'object')
    .map((model: any) => ({
      id: String(model.id || crypto.randomUUID()),
      name: String(model.name || model.provider || model.model || 'AI Model'),
      baseUrl: String(model.baseUrl || ''),
      apiKey: String(model.apiKey || ''),
      model: String(model.model || ''),
      enabled: model.enabled !== false,
    }))

  const normalizedModels = models.length ? models : defaults.models
  const defaultModelId = normalizedModels.some((model) => model.id === raw.defaultModelId)
    ? raw.defaultModelId
    : normalizedModels[0]?.id

  return {
    models: normalizedModels,
    defaultModelId,
    lastUpdated: raw.lastUpdated ?? Date.now(),
  }
}
