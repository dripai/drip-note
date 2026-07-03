<script setup lang="ts">
import { computed, ref } from 'vue'
import IconRenderer from './IconRenderer.vue'
import { getIconOption, iconCategories, iconOptions, type IconCategoryKey } from './iconMap'

const props = defineProps<{ value?: string }>()
const emit = defineEmits<{ 'update:value': [value?: string] }>()

const messages = {
  choose: '\u8bf7\u9009\u62e9\u56fe\u6807',
  clear: '\u6e05\u7a7a',
  search: '\u641c\u7d22\u56fe\u6807',
  empty: '\u6682\u65e0\u56fe\u6807',
}

const keyword = ref('')
const open = ref(false)
const activeCategory = ref<IconCategoryKey>('common')

const selectedOption = computed(() => getIconOption(props.value))

const categoryOptions = computed(() => {
  if (activeCategory.value === 'all') return iconOptions
  return iconOptions.filter((item) => item.category === activeCategory.value)
})

const filtered = computed(() => {
  const text = keyword.value.trim().toLowerCase()
  const source = text ? iconOptions : categoryOptions.value
  if (!text) return source
  return source.filter((item) => {
    return (
      item.label.toLowerCase().includes(text) ||
      item.value.toLowerCase().includes(text) ||
      item.component.toLowerCase().includes(text)
    )
  })
})

function selectIcon(value: string) {
  emit('update:value', value)
  open.value = false
}

function clearIcon() {
  emit('update:value', undefined)
}
</script>

<template>
  <a-popover
    :open="open"
    trigger="click"
    placement="bottomLeft"
    overlay-class-name="icon-select-popover"
    @open-change="open = $event"
  >
    <button type="button" class="icon-select-trigger">
      <span v-if="props.value" class="icon-select-current">
        <IconRenderer :icon="props.value" />
        <span class="icon-select-name">{{ selectedOption?.label ?? props.value }}</span>
      </span>
      <span v-else class="icon-select-placeholder">{{ messages.choose }}</span>
      <a-button
        v-if="props.value"
        type="text"
        size="small"
        class="icon-select-clear"
        @click.stop="clearIcon"
      >
        {{ messages.clear }}
      </a-button>
    </button>
    <template #content>
      <div class="icon-select-panel">
        <a-input v-model:value="keyword" allow-clear :placeholder="messages.search" class="icon-search" />
        <div class="icon-body">
          <div class="icon-category-list">
            <button
              v-for="category in iconCategories"
              :key="category.key"
              type="button"
              class="icon-category"
              :class="{ active: category.key === activeCategory, all: category.key === 'all' }"
              @click="activeCategory = category.key"
            >
              {{ category.label }}
            </button>
          </div>
          <div class="icon-results">
            <div class="icon-grid-wrap">
              <div class="icon-grid">
                <button
                  v-for="item in filtered"
                  :key="item.value"
                  type="button"
                  class="icon-cell"
                  :class="{ selected: item.value === props.value }"
                  :title="item.label + ' (' + item.value + ')'"
                  @click="selectIcon(item.value)"
                >
                  <IconRenderer :icon="item.value" />
                </button>
              </div>
              <a-empty v-if="!filtered.length" :description="messages.empty" />
            </div>
          </div>
        </div>
      </div>
    </template>
  </a-popover>
</template>

<style scoped>
.icon-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  height: 32px;
  padding: 0 8px;
  cursor: pointer;
  background: #fff;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
}

.icon-select-trigger:hover {
  border-color: #1677ff;
}

.icon-select-current {
  display: inline-flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
}

.icon-select-name {
  overflow: hidden;
  color: #1f2937;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.icon-select-placeholder {
  color: #9ca3af;
}

.icon-select-clear {
  flex: 0 0 auto;
  padding-inline: 4px;
}

.icon-select-panel {
  width: 560px;
}

.icon-search {
  margin-bottom: 10px;
}

.icon-body {
  display: flex;
  height: 360px;
  align-items: stretch;
  gap: 10px;
}

.icon-category-list {
  display: flex;
  flex: 0 0 76px;
  flex-direction: column;
  gap: 4px;
  padding-right: 8px;
  border-right: 1px solid #e5e7eb;
}

.icon-category {
  height: 30px;
  padding: 0 6px;
  overflow: hidden;
  color: #475569;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
}

.icon-category:hover {
  color: #1677ff;
  background: #f5f7fb;
}

.icon-category.active {
  color: #1677ff;
  font-weight: 600;
  background: #e6f4ff;
  border-color: #91caff;
}

.icon-category.all {
  margin-top: auto;
}

.icon-results {
  flex: 1;
  min-width: 0;
}

.icon-grid-wrap {
  height: 360px;
  overflow-y: auto;
  padding-right: 2px;
}

.icon-grid {
  display: grid;
  grid-template-columns: repeat(11, 34px);
  gap: 6px;
  align-content: start;
}

.icon-cell {
  display: inline-flex;
  width: 34px;
  height: 34px;
  align-items: center;
  justify-content: center;
  color: #475569;
  cursor: pointer;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
}

.icon-cell:hover {
  color: #1677ff;
  border-color: #1677ff;
}

.icon-cell.selected {
  color: #1677ff;
  background: #e6f4ff;
  border-color: #1677ff;
}

:global(html[data-theme='dark']) .icon-select-trigger {
  color: #d4d4d4;
  background: #1e1e1e;
  border-color: #3c3c3c;
}

:global(html[data-theme='dark']) .icon-select-name {
  color: #d4d4d4;
}

:global(html[data-theme='dark']) .icon-select-panel {
  color: #d4d4d4;
}

:global(html[data-theme='dark']) .icon-category-list {
  border-right-color: #2d2d2d;
}

:global(html[data-theme='dark']) .icon-category {
  color: #c5c5c5;
}

:global(html[data-theme='dark']) .icon-category:hover {
  color: #9cdcfe;
  background: #252526;
}

:global(html[data-theme='dark']) .icon-category.active {
  color: #ffffff;
  background: #094771;
  border-color: #007acc;
}

:global(html[data-theme='dark']) .icon-cell {
  color: #c5c5c5;
  background: #1e1e1e;
  border-color: #3c3c3c;
}

:global(html[data-theme='dark']) .icon-cell:hover,
:global(html[data-theme='dark']) .icon-cell.selected {
  color: #ffffff;
  background: #094771;
  border-color: #007acc;
}
</style>
