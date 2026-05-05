<script setup lang="ts">
import { onMounted } from 'vue'
import { useArtistsStore } from '@/stores/artists'
import type { Artist } from '@/api/types'

const store = useArtistsStore()
onMounted(() => store.fetchAll())

function formatLifespan(a: Artist): string {
  if (a.birth_year === null && a.death_year === null) return ''
  const birth = a.birth_year ?? '?'
  const death = a.death_year ?? 'present'
  return `${birth} – ${death}`
}
</script>

<template>
  <div>
    <div class="flex items-baseline justify-between mb-6">
      <h1 class="text-2xl font-semibold text-stone-900">Artists</h1>
      <span class="text-sm text-stone-500">
        {{ store.items.length }} {{ store.items.length === 1 ? 'artist' : 'artists' }}
      </span>
    </div>

    <div v-if="store.loading" class="text-stone-500">Loading…</div>

    <div
      v-else-if="store.error"
      class="p-4 border border-red-200 bg-red-50 rounded-lg text-red-800"
    >
      {{ store.error }}
    </div>

    <div v-else-if="store.items.length === 0" class="text-stone-500">
      No artists yet. Create one via Insomnia.
    </div>

    <ul v-else class="grid gap-3">
      <li
        v-for="artist in store.items"
        :key="artist.id"
        class="p-4 border border-stone-200 rounded-lg bg-white"
      >
        <h3 class="font-semibold text-stone-900">{{ artist.display_name }}</h3>
        <p class="text-sm text-stone-600 mt-1">
          <span v-if="formatLifespan(artist)">{{ formatLifespan(artist) }}</span>
          <span v-if="formatLifespan(artist) && artist.region"> · </span>
          <span v-if="artist.region">{{ artist.region }}</span>
        </p>
        <p
          v-if="artist.biography"
          class="text-sm text-stone-500 mt-2 line-clamp-2"
        >
          {{ artist.biography }}
        </p>
      </li>
    </ul>
  </div>
</template>
