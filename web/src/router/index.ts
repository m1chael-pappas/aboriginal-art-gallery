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
    path: '/artists/new',
    name: 'artists.new',
    component: () => import('@/views/ArtistCreateView.vue'),
  },
  {
    path: '/artists/:id',
    name: 'artists.detail',
    component: () => import('@/views/ArtistDetailView.vue'),
    props: true,
  },
  {
    path: '/artists/:id/edit',
    name: 'artists.edit',
    component: () => import('@/views/ArtistEditView.vue'),
    props: true,
  },

  {
    path: '/artifacts',
    name: 'artifacts',
    component: () => import('@/views/ArtifactsView.vue'),
  },
  {
    path: '/artifacts/new',
    name: 'artifacts.new',
    component: () => import('@/views/ArtifactCreateView.vue'),
  },
  {
    path: '/artifacts/:id',
    name: 'artifacts.detail',
    component: () => import('@/views/ArtifactDetailView.vue'),
    props: true,
  },
  {
    path: '/artifacts/:id/edit',
    name: 'artifacts.edit',
    component: () => import('@/views/ArtifactEditView.vue'),
    props: true,
  },

  {
    path: '/tribes',
    name: 'tribes',
    component: () => import('@/views/TribesView.vue'),
  },
  {
    path: '/tribes/new',
    name: 'tribes.new',
    component: () => import('@/views/TribeCreateView.vue'),
  },
  {
    path: '/tribes/:id',
    name: 'tribes.detail',
    component: () => import('@/views/TribeDetailView.vue'),
    props: true,
  },
  {
    path: '/tribes/:id/edit',
    name: 'tribes.edit',
    component: () => import('@/views/TribeEditView.vue'),
    props: true,
  },
]

export default createRouter({
  history: createWebHistory(),
  routes,
})
