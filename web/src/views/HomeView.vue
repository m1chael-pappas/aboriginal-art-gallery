<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { useArtistsStore } from '@/stores/artists'
import { useArtifactsStore } from '@/stores/artifacts'
import { useTribesStore } from '@/stores/tribes'

const artists = useArtistsStore()
const artifacts = useArtifactsStore()
const tribes = useTribesStore()

onMounted(() => {
  artists.fetchAll()
  artifacts.fetchAll()
  tribes.fetchAll()
})

const routes = computed(() => [
  {
    path: '/artists',
    label: 'artists',
    count: artists.items.length,
    description: 'individual makers, biographical entries',
  },
  {
    path: '/artifacts',
    label: 'artifacts',
    count: artifacts.items.length,
    description: 'objects, paintings, and ceremonial pieces',
  },
  {
    path: '/tribes',
    label: 'tribes',
    count: tribes.items.length,
    description: 'language groups & Country',
  },
])
</script>

<template>
  <div>
    <div class="grid lg:grid-cols-[1fr_280px] gap-10 mb-12">
      <div>
        <div
          class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3"
        >
          001 / about
        </div>
        <h1
          class="text-3xl font-medium text-ink leading-tight max-w-2xl"
        >
          A digital archive of Aboriginal art, artifacts and the peoples behind them.
        </h1>
        <p
          class="mt-5 text-sm text-muted max-w-xl leading-relaxed"
        >
          A curated set of works from across the continent — paintings, sculptures,
          and traditional pieces — alongside the artists and tribes that produced them.
        </p>
      </div>
      <div
        class="border border-line aspect-[4/3] flex items-center justify-center"
      >
        <span class="font-mono text-[10px] tracking-wider uppercase text-muted">
          archive photo
        </span>
      </div>
    </div>

    <div>
      <div
        class="grid grid-cols-[1fr_80px_2fr_40px] gap-3 pb-2 border-b border-line text-[10px] font-mono uppercase tracking-wider text-muted"
      >
        <span>route</span>
        <span>count</span>
        <span>description</span>
        <span></span>
      </div>
      <RouterLink
        v-for="r in routes"
        :key="r.path"
        :to="r.path"
        class="grid grid-cols-[1fr_80px_2fr_40px] gap-3 py-3 border-b border-dashed border-line items-center text-sm hover:bg-ochre/5 transition-colors"
      >
        <span class="font-mono text-ink">/{{ r.label }}</span>
        <span class="font-mono text-ink">{{ r.count }}</span>
        <span class="text-muted">{{ r.description }}</span>
        <span class="font-mono text-ochre">→</span>
      </RouterLink>
    </div>
  </div>
</template>
