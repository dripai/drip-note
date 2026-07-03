import { invoke } from '@tauri-apps/api/core'
import { useConfigStore } from '@/stores/configStore'
import type { AiModel } from './ai.types'

export type AiRequestResult<T = unknown> = {
  model: AiModel
  data: T
}

export function getDefaultAiModel(modelId?: string): AiModel | null {
  const cfg = useConfigStore().ensureAiConfig()
  const preferredId = modelId || cfg.defaultModelId
  const preferred = cfg.models.find((model) => model.id === preferredId && model.enabled)
  if (preferred) return preferred
  return cfg.models.find((model) => model.enabled) ?? null
}

export async function invokeAiRequest<T = unknown>(
  payload: Record<string, unknown>,
  modelId?: string,
): Promise<AiRequestResult<T>> {
  const model = getDefaultAiModel(modelId)
  if (!model) throw new Error('No enabled AI model configured')
  if (!model.baseUrl || !model.apiKey || !model.model) throw new Error(`AI model "${model.name}" is incomplete`)

  const data = await invoke<T>('ai_route_request', {
    payload,
    model: {
      id: model.id,
      provider: 'openai',
      baseUrl: model.baseUrl,
      apiKey: model.apiKey,
      model: model.model,
    },
  })

  return { model, data }
}
