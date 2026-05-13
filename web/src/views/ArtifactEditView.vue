<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import ArtifactForm from '@/components/ArtifactForm.vue'
import { useArtifactsStore } from '@/stores/artifacts'
import type { Artifact, ArtifactInput } from '@/api/types'

const props = defineProps<{ id: string }>()

const router = useRouter()
const artifacts = useArtifactsStore()

const initial = ref<ArtifactInput | null>(null)
const loadError = ref<string | null>(null)
const submitting = ref(false)
const submitError = ref<string | null>(null)

onMounted(async () => {
  const cached = artifacts.items.find((a) => a.id === props.id)
  if (cached) {
    initial.value = toInput(cached)
    return
  }
  try {
    initial.value = toInput(await artifacts.findById(props.id))
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : 'unknown error'
  }
})

function toInput(a: Artifact): ArtifactInput {
  return {
    title: a.title,
    artist_id: a.artist_id,
    art_type: a.art_type,
    art_style: a.art_style,
    medium: a.medium,
    year_created: a.year_created,
    height_cm: a.height_cm,
    width_cm: a.width_cm,
    depth_cm: a.depth_cm,
    description: a.description,
  }
}

async function onSubmit(input: ArtifactInput) {
  submitting.value = true
  submitError.value = null
  try {
    await artifacts.update(props.id, input)
    router.push({ name: 'artifacts.detail', params: { id: props.id } })
  } catch (e) {
    submitError.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    submitting.value = false
  }
}

function onCancel() {
  router.push({ name: 'artifacts.detail', params: { id: props.id } })
}
</script>

<template>
  <div>
    <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
      artifacts / edit
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Edit artifact</h1>

    <div v-if="loadError" class="p-4 border border-ochre/40 bg-ochre/5 text-sm text-ink">
      {{ loadError }}
    </div>
    <div v-else-if="!initial" class="text-sm text-muted">Loading…</div>
    <ArtifactForm
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
