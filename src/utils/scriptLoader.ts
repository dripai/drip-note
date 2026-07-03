export function loadScript(url: string): Promise<void> {
  return new Promise((resolve, reject) => {
    if (document.querySelector(`script[src="${url}"]`)) {
      resolve()
      return
    }
    
    const script = document.createElement('script')
    script.src = url
    script.onload = () => resolve()
    script.onerror = () => reject(new Error(`Failed to load script: ${url}`))
    document.head.appendChild(script)
  })
}

// Helper to expose Vue context to plugins
export function exposeContext(context: Record<string, any>) {
  ;(window as any).DripContext = {
    ...(window as any).DripContext,
    ...context
  }
}