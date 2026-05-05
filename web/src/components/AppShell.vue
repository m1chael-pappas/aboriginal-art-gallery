<script setup lang="ts">
import { RouterLink } from 'vue-router'

const navItems = [
  { to: '/', label: 'Home' },
  { to: '/artists', label: 'Artists' },
  { to: '/artifacts', label: 'Artifacts' },
  { to: '/tribes', label: 'Tribes' },
]
</script>

<template>
  <div class="min-h-screen flex flex-col bg-stone-50 text-stone-900">
    <header class="border-b border-stone-200 bg-white">
      <div class="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
        <RouterLink to="/" class="text-xl font-semibold tracking-tight">
          Aboriginal Art Gallery
        </RouterLink>
        <nav class="flex items-center gap-1">
          <RouterLink v-for="item in navItems" :key="item.to" :to="item.to"
            class="px-3 py-1.5 rounded-md text-sm text-stone-600 hover:text-stone-900 hover:bg-stone-100 transition-colors"
            active-class="text-stone-900 bg-stone-100">
            {{ item.label }}
          </RouterLink>
        </nav>
      </div>
    </header>
    <main class="flex-1">
      <div class="max-w-6xl mx-auto px-6 py-8">
        <slot />
      </div>
    </main>
  </div>
</template>
