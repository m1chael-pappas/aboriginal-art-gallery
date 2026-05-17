<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

// `users` is admin-only and the route enforces that with a guard, so it's
// only worth showing in the nav when the current session is an admin.
const navItems = computed(() => {
  const items = [
    { to: '/artists', label: 'artists' },
    { to: '/artifacts', label: 'artifacts' },
    { to: '/tribes', label: 'tribes' },
  ]
  if (auth.isAdmin) items.push({ to: '/users', label: 'users' })
  return items
})

const breadcrumb = computed(() => {
  const raw = route.name === 'home' ? 'index' : String(route.name ?? 'index')
  const slug = raw.replace(/\./g, ' / ')
  return `[ aboriginal_art / ${slug} ]`
})

function onSignOut() {
  auth.logout()
  router.push({ name: 'home' })
}
</script>

<template>
  <div class="min-h-screen flex flex-col">
    <header class="border-b border-line">
      <div class="max-w-6xl mx-auto px-10 py-4 flex items-center justify-between gap-6">
        <RouterLink
          to="/"
          class="font-mono text-sm text-ink hover:text-ochre transition-colors"
        >
          {{ breadcrumb }}
        </RouterLink>
        <nav class="flex items-center gap-6 font-mono text-xs">
          <RouterLink
            v-for="item in navItems"
            :key="item.to"
            :to="item.to"
            class="text-muted hover:text-ink transition-colors"
            active-class="text-ochre"
          >
            {{ item.label }}
          </RouterLink>

          <span class="text-line">·</span>

          <template v-if="auth.isAuthenticated && auth.user">
            <span class="text-muted">
              {{ auth.user.email }}
              <span v-if="auth.isAdmin" class="text-ochre">· admin</span>
            </span>
            <button
              type="button"
              @click="onSignOut"
              class="text-muted hover:text-ink transition-colors uppercase tracking-widest"
            >
              sign out
            </button>
          </template>
          <template v-else>
            <RouterLink
              :to="{ name: 'login' }"
              class="text-muted hover:text-ink transition-colors"
              active-class="text-ochre"
            >
              sign in
            </RouterLink>
          </template>
        </nav>
      </div>
    </header>

    <main class="flex-1">
      <div class="max-w-6xl mx-auto px-10 py-10">
        <slot />
      </div>
    </main>

    <footer class="border-t border-line">
      <div class="max-w-6xl mx-auto px-10 py-6">
        <div
          class="font-mono text-[10px] tracking-widest uppercase text-muted mb-2"
        >
          acknowledgement of country
        </div>
        <p class="text-xs text-muted leading-relaxed max-w-3xl">
          We acknowledge the Traditional Custodians of the lands across Australia,
          and pay our respects to Elders past and present. This archive exists in
          recognition of their continuing connection to Country, culture, and community.
        </p>
      </div>
    </footer>
  </div>
</template>
