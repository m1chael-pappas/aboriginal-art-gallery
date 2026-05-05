import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/HomeView.vue'),
  },
  {
    path: '/artists',
    name: 'artists',
    component: () => import('@/views/ArtistsView.vue'),
  },
  {
    path: '/artifacts',
    name: 'artifacts',
    component: () => import('@/views/ArtifactsView.vue'),
  },
  {
    path: '/tribes',
    name: 'tribes',
    component: () => import('@/views/TribesView.vue'),
  },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})
