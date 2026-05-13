<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useTribesStore } from '@/stores/tribes'
import { useArtistsStore } from '@/stores/artists'
import { useArtifactsStore } from '@/stores/artifacts'

const tribes = useTribesStore()
const artists = useArtistsStore()
const artifacts = useArtifactsStore()

onMounted(() => {
  tribes.fetchAll()
  artists.fetchAll()
  artifacts.fetchAll()
})

const artistsByTribe = computed(() => {
  const map = new Map<string, string[]>()
  for (const a of artists.items) {
    if (!a.tribe_id) continue
    const list = map.get(a.tribe_id) ?? []
    list.push(a.id)
    map.set(a.tribe_id, list)
  }
  return map
})

const artifactCountByArtistId = computed(() => {
  const counts = new Map<string, number>()
  for (const a of artifacts.items) {
    counts.set(a.artist_id, (counts.get(a.artist_id) ?? 0) + 1)
  }
  return counts
})

function artistsInTribe(tribeId: string): number {
  return artistsByTribe.value.get(tribeId)?.length ?? 0
}

function artifactsInTribe(tribeId: string): number {
  const ids = artistsByTribe.value.get(tribeId) ?? []
  return ids.reduce(
    (sum, artistId) => sum + (artifactCountByArtistId.value.get(artistId) ?? 0),
    0,
  )
}

function pad(n: number): string {
  return `T-${String(n).padStart(2, '0')}`
}
</script>

<template>
  <div>
    <div class="flex items-baseline justify-between mb-3">
      <div class="font-mono text-[10px] tracking-widest uppercase text-muted">
        peoples · {{ tribes.items.length }}
        {{ tribes.items.length === 1 ? 'entry' : 'entries' }}
      </div>
      <RouterLink
        :to="{ name: 'tribes.new' }"
        class="font-mono text-xs uppercase tracking-widest text-ink hover:text-ochre transition-colors"
      >
        + new
      </RouterLink>
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Tribes</h1>

    <div v-if="tribes.loading" class="text-sm text-muted">Loading…</div>

    <div
      v-else-if="tribes.error"
      class="p-4 border border-ochre/40 bg-ochre/5 text-sm text-ink"
    >
      {{ tribes.error }}
    </div>

    <div v-else-if="tribes.items.length === 0" class="text-sm text-muted">
      No tribes in the archive yet.
    </div>

    <div v-else>
      <div
        class="grid grid-cols-[60px_1.2fr_1fr_1.5fr_70px_80px] gap-3 pb-2 border-b border-line text-[10px] font-mono uppercase tracking-wider text-muted"
      >
        <span>#</span>
        <span>tribe</span>
        <span>region</span>
        <span>language</span>
        <span class="text-right">artists</span>
        <span class="text-right">artifacts</span>
      </div>
      <RouterLink
        v-for="(tribe, i) in tribes.items"
        :key="tribe.id"
        :to="{ name: 'tribes.detail', params: { id: tribe.id } }"
        class="grid grid-cols-[60px_1.2fr_1fr_1.5fr_70px_80px] gap-3 py-3 border-b border-dashed border-line items-center text-sm hover:bg-ochre/5 transition-colors"
      >
        <span class="font-mono text-xs text-muted">{{ pad(i + 1) }}</span>
        <span class="text-ink">{{ tribe.name }}</span>
        <span class="text-muted">{{ tribe.region ?? '—' }}</span>
        <span class="text-muted">{{ tribe.language_group ?? '—' }}</span>
        <span class="font-mono text-xs text-ochre text-right">
          {{ artistsInTribe(tribe.id) }}
        </span>
        <span class="font-mono text-xs text-ochre text-right">
          {{ artifactsInTribe(tribe.id) }}
        </span>
      </RouterLink>
    </div>
  </div>
</template>
