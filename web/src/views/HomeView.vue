<script setup lang="ts">
import { RouterLink } from 'vue-router'

const sections = [
  {
    name: 'Artists',
    description:
      'Aboriginal Australian artists — biographies, life dates, and tribal affiliation.',
    to: '/artists',
  },
  {
    name: 'Artifacts',
    description:
      'The artworks themselves — paintings, sculptures, weavings, and traditional pieces.',
    to: '/artifacts',
  },
  {
    name: 'Tribes',
    description:
      'Aboriginal tribes and language groups across Australia, with cultural context.',
    to: '/tribes',
  },
]
</script>

<template>
  <div class="space-y-8">
    <div>
      <h1 class="text-3xl font-bold tracking-tight text-stone-900">
        Aboriginal Art Gallery
      </h1>
      <p class="mt-2 text-stone-600">
        Browse artists, their works, and the tribal context they emerge from.
      </p>
    </div>

    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <RouterLink v-for="section in sections" :key="section.name" :to="section.to"
        class="block p-6 border border-stone-200 rounded-lg bg-white hover:shadow-sm hover:border-stone-300 transition">
        <h2 class="text-lg font-semibold text-stone-900">
          {{ section.name }}
        </h2>
        <p class="mt-1 text-sm text-stone-600">
          {{ section.description }}
        </p>
      </RouterLink>
    </div>
  </div>
</template>
