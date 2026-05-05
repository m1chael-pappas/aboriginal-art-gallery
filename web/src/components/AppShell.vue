<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

const navItems = [
  { to: '/artists', label: 'artists' },
  { to: '/artifacts', label: 'artifacts' },
  { to: '/tribes', label: 'tribes' },
]

const breadcrumb = computed(() => {
  const slug = route.name === 'home' ? 'index' : String(route.name ?? 'index')
  return `[ aboriginal_art / ${slug} ]`
})
</script>

<template>
  <div class="min-h-screen flex flex-col">
    <header class="border-b border-line">
      <div class="max-w-6xl mx-auto px-10 py-4 flex items-center justify-between">
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
