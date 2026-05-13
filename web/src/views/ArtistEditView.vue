<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import ArtistForm from '@/components/ArtistForm.vue'
import { useArtistsStore } from '@/stores/artists'
import type { Artist, ArtistInput } from '@/api/types'

const props = defineProps<{ id: string }>()

const router = useRouter()
const artists = useArtistsStore()

const initial = ref<ArtistInput | null>(null)
const loadError = ref<string | null>(null)
const submitting = ref(false)
const submitError = ref<string | null>(null)

onMounted(async () => {
  const cached = artists.items.find((a) => a.id === props.id)
  if (cached) {
    initial.value = toInput(cached)
    return
  }
  try {
    const fetched = await artists.findById(props.id)
    initial.value = toInput(fetched)
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : 'unknown error'
  }
})

function toInput(a: Artist): ArtistInput {
  return {
    display_name: a.display_name,
    birth_year: a.birth_year,
    death_year: a.death_year,
    region: a.region,
    biography: a.biography,
    tribe_id: a.tribe_id,
  }
}

async function onSubmit(input: ArtistInput) {
  submitting.value = true
  submitError.value = null
  try {
    await artists.update(props.id, input)
    router.push({ name: 'artists.detail', params: { id: props.id } })
  } catch (e) {
    submitError.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    submitting.value = false
  }
}

function onCancel() {
  router.push({ name: 'artists.detail', params: { id: props.id } })
}
</script>

<template>
  <div>
    <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
      artists / edit
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Edit artist</h1>

    <div v-if="loadError" class="p-4 border border-ochre/40 bg-ochre/5 text-sm text-ink">
      {{ loadError }}
    </div>
    <div v-else-if="!initial" class="text-sm text-muted">Loading…</div>
    <ArtistForm
      v-else
      :initial="initial"
      submit-label="save"
      :submitting="submitting"
      :error="submitError"
      @submit="onSubmit"
      @cancel="onCancel"
    />
  </div>
</template>
