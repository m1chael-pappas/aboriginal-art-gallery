<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import TribeForm from '@/components/TribeForm.vue'
import { useTribesStore } from '@/stores/tribes'
import type { TribeInput } from '@/api/types'

const router = useRouter()
const tribes = useTribesStore()

const submitting = ref(false)
const error = ref<string | null>(null)

const initial: TribeInput = {
  name: '',
  region: null,
  language_group: null,
  description: null,
}

async function onSubmit(input: TribeInput) {
  submitting.value = true
  error.value = null
  try {
    const created = await tribes.create(input)
    router.push({ name: 'tribes.detail', params: { id: created.id } })
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    submitting.value = false
  }
}

function onCancel() {
  router.push({ name: 'tribes' })
}
</script>

<template>
  <div>
    <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
      tribes / new
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">New tribe</h1>

    <TribeForm
      :initial="initial"
      submit-label="create"
      :submitting="submitting"
      :error="error"
      @submit="onSubmit"
      @cancel="onCancel"
    />
  </div>
</template>
