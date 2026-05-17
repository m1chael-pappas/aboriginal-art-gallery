<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { useArtifactsStore } from '@/stores/artifacts'
import { useArtistsStore } from '@/stores/artists'
import { useAuthStore } from '@/stores/auth'
import type { Artifact } from '@/api/types'

const props = defineProps<{ id: string }>()

const router = useRouter()
const artifacts = useArtifactsStore()
const artists = useArtistsStore()
const auth = useAuthStore()

const artifact = ref<Artifact | null>(null)
const loadError = ref<string | null>(null)
const deleting = ref(false)

onMounted(async () => {
  if (artists.items.length === 0) artists.fetchAll()

  const cached = artifacts.items.find((a) => a.id === props.id)
  if (cached) {
    artifact.value = cached
    return
  }
  try {
    artifact.value = await artifacts.findById(props.id)
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : 'unknown error'
  }
})

const artist = computed(() => {
  if (!artifact.value) return null
  return artists.items.find((a) => a.id === artifact.value!.artist_id) ?? null
})

const dimensions = computed(() => {
  if (!artifact.value) return null
  const { height_cm, width_cm, depth_cm } = artifact.value
  if (height_cm === null && width_cm === null && depth_cm === null) return null
  const parts = [height_cm, width_cm, depth_cm].map((d) => (d === null ? '?' : d))
  return `${parts[0]} × ${parts[1]}${depth_cm !== null ? ` × ${parts[2]}` : ''} cm`
})

async function onDelete() {
  if (!artifact.value) return
  if (!window.confirm(`Delete "${artifact.value.title}"? This cannot be undone.`)) {
    return
  }
  deleting.value = true
  try {
    await artifacts.remove(artifact.value.id)
    router.push({ name: 'artifacts' })
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
    <div v-else-if="!artifact" class="text-sm text-muted">Loading…</div>

    <template v-else>
      <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
        artifacts / detail
      </div>
      <div class="flex items-baseline justify-between mb-8 gap-6 flex-wrap">
        <h1 class="text-3xl font-medium text-ink">{{ artifact.title }}</h1>
        <div class="flex items-center gap-5 font-mono text-xs uppercase tracking-widest">
          <template v-if="auth.isAdmin">
            <RouterLink
              :to="{ name: 'artifacts.edit', params: { id: artifact.id } }"
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
          </template>
          <RouterLink
            :to="{ name: 'artifacts' }"
            class="text-muted hover:text-ink transition-colors"
          >
            ← back
          </RouterLink>
        </div>
      </div>

      <div class="grid lg:grid-cols-[280px_1fr] gap-10 mb-12">
        <div
          class="aspect-[4/3] border border-line flex items-center justify-center"
        >
          <span class="font-mono text-[10px] tracking-wider uppercase text-muted">
            image
          </span>
        </div>

        <dl class="grid grid-cols-[140px_1fr] gap-x-6 gap-y-3 text-sm">
          <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
            artist
          </dt>
          <dd class="text-ink">
            <RouterLink
              v-if="artist"
              :to="{ name: 'artists.detail', params: { id: artist.id } }"
              class="hover:text-ochre transition-colors"
            >
              {{ artist.display_name }}
            </RouterLink>
            <span v-else>-</span>
          </dd>

          <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
            year
          </dt>
          <dd class="font-mono text-ink">{{ artifact.year_created ?? '-' }}</dd>

          <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
            type
          </dt>
          <dd class="text-ink">{{ artifact.art_type ?? '-' }}</dd>

          <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
            style
          </dt>
          <dd class="text-ink">{{ artifact.art_style ?? '-' }}</dd>

          <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
            medium
          </dt>
          <dd class="text-ink">{{ artifact.medium ?? '-' }}</dd>

          <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
            dimensions
          </dt>
          <dd class="font-mono text-ink">{{ dimensions ?? '-' }}</dd>

          <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
            description
          </dt>
          <dd class="text-ink leading-relaxed whitespace-pre-line">
            {{ artifact.description ?? '-' }}
          </dd>
        </dl>
      </div>
    </template>
  </div>
</template>
