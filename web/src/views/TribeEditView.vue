<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import TribeForm from '@/components/TribeForm.vue'
import { useTribesStore } from '@/stores/tribes'
import type { Tribe, TribeInput } from '@/api/types'

const props = defineProps<{ id: string }>()

const router = useRouter()
const tribes = useTribesStore()

const initial = ref<TribeInput | null>(null)
const loadError = ref<string | null>(null)
const submitting = ref(false)
const submitError = ref<string | null>(null)

onMounted(async () => {
  const cached = tribes.items.find((t) => t.id === props.id)
  if (cached) {
    initial.value = toInput(cached)
    return
  }
  try {
    initial.value = toInput(await tribes.findById(props.id))
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : 'unknown error'
  }
})

function toInput(t: Tribe): TribeInput {
  return {
    name: t.name,
    region: t.region,
    language_group: t.language_group,
    description: t.description,
  }
}

async function onSubmit(input: TribeInput) {
  submitting.value = true
  submitError.value = null
  try {
    await tribes.update(props.id, input)
    router.push({ name: 'tribes.detail', params: { id: props.id } })
  } catch (e) {
    submitError.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    submitting.value = false
  }
}

function onCancel() {
  router.push({ name: 'tribes.detail', params: { id: props.id } })
}
</script>

<template>
  <div>
    <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
      tribes / edit
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Edit tribe</h1>

    <div v-if="loadError" class="p-4 border border-ochre/40 bg-ochre/5 text-sm text-ink">
      {{ loadError }}
    </div>
    <div v-else-if="!initial" class="text-sm text-muted">Loading…</div>
    <TribeForm
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
