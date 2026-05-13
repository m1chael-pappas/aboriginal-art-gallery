<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useArtifactsStore } from '@/stores/artifacts'
import { useArtistsStore } from '@/stores/artists'
import { useTribesStore } from '@/stores/tribes'

const artifacts = useArtifactsStore()
const artists = useArtistsStore()
const tribes = useTribesStore()

onMounted(() => {
  artifacts.fetchAll()
  artists.fetchAll()
  tribes.fetchAll()
})

const artistTribeId = computed(() => {
  const map = new Map<string, string | null>()
  for (const a of artists.items) map.set(a.id, a.tribe_id)
  return map
})

const tribeNameById = computed(() => {
  const map = new Map<string, string>()
  for (const t of tribes.items) map.set(t.id, t.name)
  return map
})

function tribeOf(artistId: string): string {
  const tid = artistTribeId.value.get(artistId)
  if (!tid) return ''
  return tribeNameById.value.get(tid) ?? ''
}

function pad(n: number): string {
  return `A-${String(n).padStart(3, '0')}`
}
</script>

<template>
  <div>
    <div class="flex items-baseline justify-between mb-3">
      <div class="font-mono text-[10px] tracking-widest uppercase text-muted">
        collection · {{ artifacts.items.length }}
        {{ artifacts.items.length === 1 ? 'object' : 'objects' }}
      </div>
      <RouterLink
        :to="{ name: 'artifacts.new' }"
        class="font-mono text-xs uppercase tracking-widest text-ink hover:text-ochre transition-colors"
      >
        + new
      </RouterLink>
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Artifacts</h1>

    <div v-if="artifacts.loading" class="text-sm text-muted">Loading…</div>

    <div
      v-else-if="artifacts.error"
      class="p-4 border border-ochre/40 bg-ochre/5 text-sm text-ink"
    >
      {{ artifacts.error }}
    </div>

    <div v-else-if="artifacts.items.length === 0" class="text-sm text-muted">
      No artifacts in the archive yet.
    </div>

    <div v-else class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4">
      <RouterLink
        v-for="(artifact, i) in artifacts.items"
        :key="artifact.id"
        :to="{ name: 'artifacts.detail', params: { id: artifact.id } }"
        class="border border-dashed border-line p-3 flex flex-col gap-2 hover:bg-ochre/5 transition-colors"
      >
        <div
          class="aspect-[4/3] border border-line flex items-center justify-center"
        >
          <span class="font-mono text-[10px] text-muted">image</span>
        </div>
        <div class="font-mono text-[11px] text-ochre">{{ pad(i + 1) }}</div>
        <div class="text-sm font-medium text-ink leading-snug">
          {{ artifact.title }}
        </div>
        <div class="font-mono text-[10px] text-muted">
          <span v-if="tribeOf(artifact.artist_id)">{{ tribeOf(artifact.artist_id) }}</span>
          <span v-if="tribeOf(artifact.artist_id) && artifact.year_created"> · </span>
          <span v-if="artifact.year_created">{{ artifact.year_created }}</span>
        </div>
      </RouterLink>
    </div>
  </div>
</template>
