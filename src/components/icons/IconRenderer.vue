<script setup lang="ts">
import { computed, defineComponent, h } from 'vue'
import { iconMap, type IconKey } from './iconMap'

const props = defineProps<{ icon?: string }>()

const component = computed(() => {
  if (!props.icon) return null
  const mapped = iconMap[props.icon as IconKey]
  if (mapped) return mapped
  return defineComponent({
    name: 'TextIconFallback',
    setup: () => () => h('span', { class: 'text-icon-fallback' }, props.icon?.slice(0, 2)),
  })
})
</script>

<template>
  <component :is="component" v-if="component" />
</template>

<style scoped>
.text-icon-fallback {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1em;
  font-size: 0.85em;
  font-weight: 600;
  line-height: 1;
}
</style>
