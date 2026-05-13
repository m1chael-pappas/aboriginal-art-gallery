<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { useTribesStore } from '@/stores/tribes'
import { useArtistsStore } from '@/stores/artists'
import { useArtifactsStore } from '@/stores/artifacts'
import { useAuthStore } from '@/stores/auth'
import type { Tribe } from '@/api/types'

const props = defineProps<{ id: string }>()

const router = useRouter()
const tribes = useTribesStore()
const artists = useArtistsStore()
const artifacts = useArtifactsStore()
const auth = useAuthStore()

const tribe = ref<Tribe | null>(null)
const loadError = ref<string | null>(null)
const deleting = ref(false)

onMounted(async () => {
  if (artists.items.length === 0) artists.fetchAll()
  if (artifacts.items.length === 0) artifacts.fetchAll()

  const cached = tribes.items.find((t) => t.id === props.id)
  if (cached) {
    tribe.value = cached
    return
  }
  try {
    tribe.value = await tribes.findById(props.id)
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : 'unknown error'
  }
})

const membersOfTribe = computed(() => {
  if (!tribe.value) return []
  return artists.items.filter((a) => a.tribe_id === tribe.value!.id)
})

const artifactsByMember = computed(() => {
  const counts = new Map<string, number>()
  for (const a of artifacts.items) {
    counts.set(a.artist_id, (counts.get(a.artist_id) ?? 0) + 1)
  }
  return counts
})

// The OpenAPI spec types `territory` as a generic `Object`, so on the FE
// we see it as `Record<string, unknown> | null`. Read the GeoJSON members
// through narrow guards rather than asserting a concrete shape.
type GeoJsonGeometry = {
  type?: unknown
  coordinates?: unknown
}

const territory = computed<GeoJsonGeometry | null>(() => {
  const t = tribe.value?.territory
  return t ? (t as GeoJsonGeometry) : null
})

const territoryType = computed<string | null>(() => {
  const t = territory.value?.type
  return typeof t === 'string' ? t : null
})

const polygonCount = computed<number | null>(() => {
  const coords = territory.value?.coordinates
  if (!Array.isArray(coords)) return null
  if (territoryType.value === 'MultiPolygon') return coords.length
  if (territoryType.value === 'Polygon') return 1
  return null
})

const territoryPretty = computed<string>(() => {
  return territory.value ? JSON.stringify(territory.value, null, 2) : ''
})

async function onDelete() {
  if (!tribe.value) return
  const memberCount = membersOfTribe.value.length
  const extra =
    memberCount > 0
      ? `\n\n${memberCount} ${memberCount === 1 ? 'artist references' : 'artists reference'} this tribe.`
      : ''
  if (!window.confirm(`Delete "${tribe.value.name}"?${extra}`)) {
    return
  }
  deleting.value = true
  try {
    await tribes.remove(tribe.value.id)
    router.push({ name: 'tribes' })
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
    <div v-else-if="!tribe" class="text-sm text-muted">Loading…</div>

    <template v-else>
      <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
        tribes / detail
      </div>
      <div class="flex items-baseline justify-between mb-8 gap-6 flex-wrap">
        <h1 class="text-3xl font-medium text-ink">{{ tribe.name }}</h1>
        <div class="flex items-center gap-5 font-mono text-xs uppercase tracking-widest">
          <template v-if="auth.isAdmin">
            <RouterLink
              :to="{ name: 'tribes.edit', params: { id: tribe.id } }"
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
            :to="{ name: 'tribes' }"
            class="text-muted hover:text-ink transition-colors"
          >
            ← back
          </RouterLink>
        </div>
      </div>

      <dl class="grid grid-cols-[140px_1fr] gap-x-6 gap-y-3 text-sm max-w-3xl mb-12">
        <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">region</dt>
        <dd class="text-ink">{{ tribe.region ?? '—' }}</dd>

        <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
          language group
        </dt>
        <dd class="text-ink">{{ tribe.language_group ?? '—' }}</dd>

        <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
          description
        </dt>
        <dd class="text-ink leading-relaxed whitespace-pre-line">
          {{ tribe.description ?? '—' }}
        </dd>

        <dt class="font-mono text-[10px] tracking-widest uppercase text-muted pt-1">
          territory
        </dt>
        <dd class="text-ink">
          <template v-if="territory">
            <span class="font-mono text-xs text-ochre">{{ territoryType ?? 'set' }}</span>
            <span v-if="polygonCount !== null" class="font-mono text-xs text-muted">
              · {{ polygonCount }} {{ polygonCount === 1 ? 'polygon' : 'polygons' }}
            </span>
          </template>
          <span v-else class="text-muted">—</span>
        </dd>
      </dl>

      <details v-if="territory" class="mb-12 border border-dashed border-line">
        <summary
          class="font-mono text-[10px] tracking-widest uppercase text-muted px-4 py-3 cursor-pointer hover:text-ink transition-colors"
        >
          view GeoJSON
        </summary>
        <pre
          class="px-4 py-3 border-t border-dashed border-line font-mono text-[11px] text-ink overflow-x-auto leading-relaxed"
        >{{ territoryPretty }}</pre>
      </details>

      <div>
        <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
          artists · {{ membersOfTribe.length }}
        </div>
        <div v-if="membersOfTribe.length === 0" class="text-sm text-muted">
          No artists attributed to this tribe yet.
        </div>
        <div v-else class="border-t border-line">
          <RouterLink
            v-for="a in membersOfTribe"
            :key="a.id"
            :to="{ name: 'artists.detail', params: { id: a.id } }"
            class="grid grid-cols-[1fr_80px_40px] gap-3 py-3 border-b border-dashed border-line items-center text-sm hover:bg-ochre/5 transition-colors"
          >
            <span class="text-ink">{{ a.display_name }}</span>
            <span class="font-mono text-xs text-ochre text-right">
              {{ artifactsByMember.get(a.id) ?? 0 }} works
            </span>
            <span class="font-mono text-ochre text-right">→</span>
          </RouterLink>
        </div>
      </div>
    </template>
  </div>
</template>
