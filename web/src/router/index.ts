import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/HomeView.vue'),
  },

  // Auth
  {
    path: '/login',
    name: 'login',
    component: () => import('@/views/LoginView.vue'),
  },
  {
    path: '/register',
    name: 'register',
    component: () => import('@/views/RegisterView.vue'),
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
    meta: { requiresAdmin: true },
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
    meta: { requiresAdmin: true },
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
    meta: { requiresAdmin: true },
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
    meta: { requiresAdmin: true },
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
    meta: { requiresAdmin: true },
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
    meta: { requiresAdmin: true },
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

// Guard for routes flagged `meta.requiresAdmin`. Unauthenticated callers go
// to /login with a redirect param; authenticated non-admins go back home —
// they shouldn't see a login page when their token is fine, just not
// privileged enough.
router.beforeEach((to) => {
  if (!to.meta.requiresAdmin) return true

  const auth = useAuthStore()
  if (!auth.isAuthenticated) {
    return { name: 'login', query: { redirect: to.fullPath } }
  }
  if (!auth.isAdmin) {
    return { name: 'home' }
  }
  return true
})

export default router
