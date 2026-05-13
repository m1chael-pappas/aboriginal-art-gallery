<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useArtistsStore } from '@/stores/artists'
import { useArtifactsStore } from '@/stores/artifacts'
import { useTribesStore } from '@/stores/tribes'
import { useAuthStore } from '@/stores/auth'

const artists = useArtistsStore()
const artifacts = useArtifactsStore()
const tribes = useTribesStore()
const auth = useAuthStore()

onMounted(() => {
  artists.fetchAll()
  artifacts.fetchAll()
  tribes.fetchAll()
})

const tribeNameById = computed(() => {
  const map = new Map<string, string>()
  for (const t of tribes.items) map.set(t.id, t.name)
  return map
})

const worksByArtistId = computed(() => {
  const counts = new Map<string, number>()
  for (const a of artifacts.items) {
    counts.set(a.artist_id, (counts.get(a.artist_id) ?? 0) + 1)
  }
  return counts
})

function era(birth: number | null, death: number | null): string {
  if (birth === null && death === null) return '—'
  return `${birth ?? '?'}–${death ?? 'present'}`
}

function pad(n: number): string {
  return String(n).padStart(3, '0')
}
</script>

<template>
  <div>
    <div class="flex items-baseline justify-between mb-3">
      <div class="font-mono text-[10px] tracking-widest uppercase text-muted">
        index · {{ artists.items.length }} {{ artists.items.length === 1 ? 'entry' : 'entries' }}
      </div>
      <RouterLink
        v-if="auth.isAdmin"
        :to="{ name: 'artists.new' }"
        class="font-mono text-xs uppercase tracking-widest text-ink hover:text-ochre transition-colors"
      >
        + new
      </RouterLink>
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Artists</h1>

    <div v-if="artists.loading" class="text-sm text-muted">Loading…</div>

    <div
      v-else-if="artists.error"
      class="p-4 border border-ochre/40 bg-ochre/5 text-sm text-ink"
    >
      {{ artists.error }}
    </div>

    <div v-else-if="artists.items.length === 0" class="text-sm text-muted">
      No artists in the archive yet.
    </div>

    <div v-else>
      <div
        class="grid grid-cols-[50px_2fr_1.2fr_1fr_60px] gap-3 pb-2 border-b border-line text-[10px] font-mono uppercase tracking-wider text-muted"
      >
        <span>#</span>
        <span>name</span>
        <span>tribe</span>
        <span>era</span>
        <span class="text-right">works</span>
      </div>
      <RouterLink
        v-for="(artist, i) in artists.items"
        :key="artist.id"
        :to="{ name: 'artists.detail', params: { id: artist.id } }"
        class="grid grid-cols-[50px_2fr_1.2fr_1fr_60px] gap-3 py-3 border-b border-dashed border-line items-center text-sm hover:bg-ochre/5 transition-colors"
      >
        <span class="font-mono text-xs text-muted">{{ pad(i + 1) }}</span>
        <span class="text-ink">{{ artist.display_name }}</span>
        <span class="text-muted">{{ tribeNameById.get(artist.tribe_id ?? '') ?? '—' }}</span>
        <span class="font-mono text-xs text-ink">
          {{ era(artist.birth_year, artist.death_year) }}
        </span>
        <span class="font-mono text-xs text-ochre text-right">
          {{ worksByArtistId.get(artist.id) ?? 0 }}
        </span>
      </RouterLink>
    </div>
  </div>
</template>
