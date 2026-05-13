<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import ArtifactForm from '@/components/ArtifactForm.vue'
import { useArtifactsStore } from '@/stores/artifacts'
import type { ArtifactInput } from '@/api/types'

const router = useRouter()
const artifacts = useArtifactsStore()

const submitting = ref(false)
const error = ref<string | null>(null)

const initial: ArtifactInput = {
  title: '',
  artist_id: '',
  art_type: null,
  art_style: null,
  medium: null,
  year_created: null,
  height_cm: null,
  width_cm: null,
  depth_cm: null,
  description: null,
}

async function onSubmit(input: ArtifactInput) {
  submitting.value = true
  error.value = null
  try {
    const created = await artifacts.create(input)
    router.push({ name: 'artifacts.detail', params: { id: created.id } })
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    submitting.value = false
  }
}

function onCancel() {
  router.push({ name: 'artifacts' })
}
</script>

<template>
  <div>
    <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
      artifacts / new
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">New artifact</h1>

    <ArtifactForm
      :initial="initial"
      submit-label="create"
      :submitting="submitting"
      :error="error"
      @submit="onSubmit"
      @cancel="onCancel"
    />
  </div>
</template>
