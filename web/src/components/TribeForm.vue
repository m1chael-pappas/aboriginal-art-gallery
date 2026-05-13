<script setup lang="ts">
import { reactive } from 'vue'
import FormField from '@/components/FormField.vue'
import type { TribeInput } from '@/api/types'

const props = defineProps<{
  initial: TribeInput
  submitLabel: string
  submitting: boolean
  error: string | null
}>()

const emit = defineEmits<{
  submit: [input: TribeInput]
  cancel: []
}>()

const form = reactive<TribeInput>({
  name: props.initial.name,
  region: props.initial.region ?? null,
  language_group: props.initial.language_group ?? null,
  description: props.initial.description ?? null,
})

function emptyToNull(v: string | null | undefined): string | null {
  if (!v) return null
  const trimmed = v.trim()
  return trimmed === '' ? null : trimmed
}

function onSubmit() {
  emit('submit', {
    name: form.name.trim(),
    region: emptyToNull(form.region),
    language_group: emptyToNull(form.language_group),
    description: emptyToNull(form.description),
  })
}

const inputCls =
  'w-full bg-paper border border-line px-3 py-2 text-sm text-ink focus:border-ochre focus:outline-none'
</script>

<template>
  <form @submit.prevent="onSubmit" class="space-y-6 max-w-2xl">
    <FormField label="name" required>
      <input v-model="form.name" required :class="inputCls" type="text" />
    </FormField>

    <FormField label="region">
      <input v-model="form.region" :class="inputCls" type="text" />
    </FormField>

    <FormField label="language group">
      <input v-model="form.language_group" :class="inputCls" type="text" />
    </FormField>

    <FormField label="description">
      <textarea v-model="form.description" :class="inputCls" rows="6" />
    </FormField>

    <div v-if="error" class="p-3 border border-ochre/40 bg-ochre/5 text-sm text-ink">
      {{ error }}
    </div>

    <div class="flex items-center gap-4 pt-2 border-t border-dashed border-line">
      <button
        type="submit"
        :disabled="submitting"
        class="font-mono text-xs uppercase tracking-widest text-ink hover:text-ochre disabled:opacity-50 transition-colors"
      >
        {{ submitting ? 'saving…' : submitLabel }}
      </button>
      <button
        type="button"
        @click="emit('cancel')"
        class="font-mono text-xs uppercase tracking-widest text-muted hover:text-ink transition-colors"
      >
        cancel
      </button>
    </div>
  </form>
</template>
