<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import ArtistForm from '@/components/ArtistForm.vue'
import { useArtistsStore } from '@/stores/artists'
import type { ArtistInput } from '@/api/types'

const router = useRouter()
const artists = useArtistsStore()

const submitting = ref(false)
const error = ref<string | null>(null)

const initial: ArtistInput = {
  display_name: '',
  birth_year: null,
  death_year: null,
  region: null,
  biography: null,
  tribe_id: null,
}

async function onSubmit(input: ArtistInput) {
  submitting.value = true
  error.value = null
  try {
    const created = await artists.create(input)
    router.push({ name: 'artists.detail', params: { id: created.id } })
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    submitting.value = false
  }
}

function onCancel() {
  router.push({ name: 'artists' })
}
</script>

<template>
  <div>
    <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
      artists / new
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">New artist</h1>

    <ArtistForm
      :initial="initial"
      submit-label="create"
      :submitting="submitting"
      :error="error"
      @submit="onSubmit"
      @cancel="onCancel"
    />
  </div>
</template>
