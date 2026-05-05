<script setup lang="ts">
import { onMounted } from 'vue'
import { useArtifactsStore } from '@/stores/artifacts'

const store = useArtifactsStore()
onMounted(() => store.fetchAll())

function formatDimensions(
  h: number | null,
  w: number | null,
  d: number | null,
): string {
  const parts = [h, w, d].filter((v): v is number => v !== null)
  if (parts.length === 0) return ''
  return `${parts.join(' × ')} cm`
}
</script>

<template>
  <div>
    <div class="flex items-baseline justify-between mb-6">
      <h1 class="text-2xl font-semibold text-stone-900">Artifacts</h1>
      <span class="text-sm text-stone-500">
        {{ store.items.length }} {{ store.items.length === 1 ? 'artifact' : 'artifacts' }}
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
      No artifacts yet. Create one via Insomnia.
    </div>

    <ul v-else class="grid gap-3">
      <li
        v-for="artifact in store.items"
        :key="artifact.id"
        class="p-4 border border-stone-200 rounded-lg bg-white"
      >
        <div class="flex items-baseline justify-between gap-3">
          <h3 class="font-semibold text-stone-900">{{ artifact.title }}</h3>
          <span
            v-if="artifact.year_created"
            class="text-sm text-stone-500 shrink-0"
          >
            {{ artifact.year_created }}
          </span>
        </div>
        <p class="text-sm text-stone-600 mt-1">
          <span v-if="artifact.art_type">{{ artifact.art_type }}</span>
          <span v-if="artifact.art_type && artifact.art_style"> · </span>
          <span v-if="artifact.art_style">{{ artifact.art_style }}</span>
        </p>
        <p v-if="artifact.medium" class="text-sm text-stone-500 mt-1">
          {{ artifact.medium }}
        </p>
        <p
          v-if="formatDimensions(artifact.height_cm, artifact.width_cm, artifact.depth_cm)"
          class="text-sm text-stone-500 mt-1"
        >
          {{ formatDimensions(artifact.height_cm, artifact.width_cm, artifact.depth_cm) }}
        </p>
      </li>
    </ul>
  </div>
</template>
