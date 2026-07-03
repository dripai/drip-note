import { convertFileSrc } from '@tauri-apps/api/core'
import type { AppConfig } from '@/services/config/config.provider'

const IMAGE_PLACEHOLDER_PREFIX = 'drip-img://'

function normalizeForwardSlashes(path: string) {
  return path.replace(/\\/g, '/')
}

function stripTrailingSlash(path: string) {
  return path.endsWith('/') ? path.slice(0, -1) : path
}

export function buildImagePlaceholder(fullPath: string, cfg: AppConfig) {
  const normalizedFull = normalizeForwardSlashes(fullPath)
  if (cfg.imagesDir) {
    const base = stripTrailingSlash(normalizeForwardSlashes(cfg.imagesDir))
    if (normalizedFull.startsWith(base + '/')) {
      const rel = normalizedFull.slice(base.length + 1)
      return `${IMAGE_PLACEHOLDER_PREFIX}${rel}`
    }
  }
  return 'file://' + normalizedFull
}

export function resolveImageSrc(value: string, cfg: AppConfig) {
  if (value.startsWith('file://')) {
    let path = value.replace(/^file:\/\//, '')
    if (/^\/[A-Za-z]:\//.test(path)) path = path.slice(1)
    return convertFileSrc(normalizeForwardSlashes(path))
  }
  if (value.startsWith(IMAGE_PLACEHOLDER_PREFIX) && cfg.imagesDir) {
    const rel = value.slice(IMAGE_PLACEHOLDER_PREFIX.length).replace(/^\/+/, '')
    const base = stripTrailingSlash(normalizeForwardSlashes(cfg.imagesDir))
    const full = `${base}/${rel}`
    return convertFileSrc(full)
  }
  return null
}

export function sanitizeHtml(html: string, cfg: AppConfig) {
  return html.replace(/<img([^>]*)>/gi, (_m, attrs) => {
    let newAttrs = attrs

    // 1. Handle src resolution
    const mSrc = attrs.match(/\bsrc=(["']?)([^"'>\s]+)\1/i)
    if (mSrc) {
      const resolved = resolveImageSrc(mSrc[2], cfg)
      if (resolved) {
        newAttrs = newAttrs.replace(mSrc[0], `src="${resolved}"`)
      }
    }

    // 2. Handle width/height from alt
    // Support Obsidian style: ![alt|100] or ![alt|100x200]
    // Support Hash style: ![alt#100] or ![alt#100x200]
    const mAlt = attrs.match(/\balt=(["']?)(.*?)\1/i)
    if (mAlt) {
      const fullAlt = mAlt[2]
      // Regex to match: desc|width, desc|widthxheight, desc#width, desc#widthxheight
      // Captures: 1=desc, 2=width, 3=height(optional)
      // The separator can be | or #
      const sizeMatch = fullAlt.match(/^(.*)[|#](\d+)(?:x(\d+))?$/)
      
      if (sizeMatch) {
        const [, desc, width, height] = sizeMatch
        
        // Update alt to remove size info
        newAttrs = newAttrs.replace(mAlt[0], `alt="${desc.trim()}"`)
        
        // Construct style
        let style = `width: ${width}px;`
        if (height) {
          style += ` height: ${height}px;`
        } else {
          // If only width is provided, ensure aspect ratio is maintained
          style += ` height: auto;`
        }

        // Inject or merge style
        const mStyle = newAttrs.match(/\bstyle=(["']?)(.*?)\1/i)
        if (mStyle) {
          newAttrs = newAttrs.replace(mStyle[0], `style="${mStyle[2]}; ${style}"`)
        } else {
          newAttrs += ` style="${style}"`
        }
      }
    }

    return `<img${newAttrs}>`
  })
}
