<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { useArtistsStore } from '@/stores/artists'
import { useTribesStore } from '@/stores/tribes'
import { useArtifactsStore } from '@/stores/artifacts'
import type { Artist } from '@/api/types'

const props = defineProps<{ id: string }>()

const router = useRouter()
const artists = useArtistsStore()
const tribes = useTribesStore()
const artifacts = useArtifactsStore()

const artist = ref<Artist | null>(null)
const loadError = ref<string | null>(null)
const deleting = ref(false)

onMounted(async () => {
  if (tribes.items.length === 0) tribes.fetchAll()
  if (artifacts.items.length === 0) artifacts.fetchAll()

  const cached = artists.items.find((a) => a.id === props.id)
  if (cached) {
    artist.value = cached
    return
  }
  try {
    artist.value = await artists.findById(props.id)
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : 'unknown error'
  }
})

const tribeName = computed(() => {
  if (!artist.value?.tribe_id) return null
  return tribes.items.find((t) => t.id === artist.value!.tribe_id)?.name ?? null
})

const works = computed(() => {
  if (!artist.value) return []
  return artifacts.items.filter((a) => a.artist_id === artist.value!.id)
})

const era = computed(() => {
  if (!artist.value) return '—'
  const { birth_year, death_year } = artist.value
  if (birth_year === null && death_year === null) return '—'
  return `${birth_year ?? '?'}–${death_year ?? 'present'}`
})

async function onDelete() {
  if (!artist.value) return
  if (!window.confirm(`Delete "${artist.value.display_name}"? This cannot be undone.`)) {
    return
  }
  deleting.value = true
  try {
    await artists.remove(artist.value.id)
    router.push({ name: 'artists' })
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : 'unknown error'
    deleting.value = false
  }
}
</script>

<template>
  <div>
    <div v-if="loadError" class="p-4 border border-ochre/40 bg-ochre/5 text-sm text-ink">
      {{ loadError }}
    </div>
    <div v-else-if="!artist" class="text-sm text-muted">Loading…</div>

    <template v-else>
      <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
        artists / detail
      </div>
      <div class="flex items-baseline justify-between mb-8 gap-6 flex-wrap">
        <h1 class="text-3xl font-medium text-ink">{{ artist.display_name }}</h1>
        <div class="flex items-center gap-5 font-mono text-xs uppercase tracking-widest">
          <RouterLink
            :to="{ name: 'artists.edit', params: { id: artist.id } }"
            class="text-ink hover:text-ochre transition-colors"
          >
            edit
          </RouterLink>
          <button
            type="button"
            :disabled="deleting"
            @click="onDelete"
            class="text-muted hover:text-ochre disabled:opacity-50 transition-colors"
          >
            {{ deleting ? 'deleting…' : 'delete' }}
          </button>
          <RouterLink
            :to="{ name: 'artists' }"
            class="text-muted hover:text-ink transition-colors"
          >
            ← back
          </RouterLink>
        </div>
      </div>

      <dl class="grid grid-cols-[140px_1fr] gap-x-6 gap-y-3 text-sm max-w-3xl mb-12">
        <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">era</dt>
        <dd class="font-mono text-ink">{{ era }}</dd>

        <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">region</dt>
        <dd class="text-ink">{{ artist.region ?? '—' }}</dd>

        <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">tribe</dt>
        <dd class="text-ink">
          {{ tribeName ?? '—' }}
        </dd>

        <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
          biography
        </dt>
        <dd class="text-ink leading-relaxed whitespace-pre-line">
          {{ artist.biography ?? '—' }}
        </dd>
      </dl>

      <div>
        <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
          works · {{ works.length }}
        </div>
        <div v-if="works.length === 0" class="text-sm text-muted">
          No artifacts attributed yet.
        </div>
        <div v-else class="border-t border-line">
          <RouterLink
            v-for="w in works"
            :key="w.id"
            :to="{ name: 'artifacts.detail', params: { id: w.id } }"
            class="grid grid-cols-[1fr_80px_40px] gap-3 py-3 border-b border-dashed border-line items-center text-sm hover:bg-ochre/5 transition-colors"
          >
            <span class="text-ink">{{ w.title }}</span>
            <span class="font-mono text-xs text-muted text-right">
              {{ w.year_created ?? '—' }}
            </span>
            <span class="font-mono text-ochre text-right">→</span>
          </RouterLink>
        </div>
      </div>
    </template>
  </div>
</template>
