import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'
import MainLayout from '../layouts/MainLayout.vue'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: MainLayout,
    children: [
      { path: '', redirect: { name: 'notes' } },
      { path: 'notes/:id?', name: 'notes', component: () => import('../views/Notes.vue') },
      { path: 'settings', name: 'settings', component: () => import('../views/settings/index.vue') },
    ],
  },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})
