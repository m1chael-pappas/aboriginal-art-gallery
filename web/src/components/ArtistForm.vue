<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import FormField from '@/components/FormField.vue'
import { useTribesStore } from '@/stores/tribes'
import type { ArtistInput } from '@/api/types'

const props = defineProps<{
  initial: ArtistInput
  submitLabel: string
  submitting: boolean
  error: string | null
}>()

const emit = defineEmits<{
  submit: [input: ArtistInput]
  cancel: []
}>()

const tribes = useTribesStore()
onMounted(() => {
  if (tribes.items.length === 0) tribes.fetchAll()
})

const form = reactive<ArtistInput>({
  display_name: props.initial.display_name,
  birth_year: props.initial.birth_year ?? null,
  death_year: props.initial.death_year ?? null,
  region: props.initial.region ?? null,
  biography: props.initial.biography ?? null,
  tribe_id: props.initial.tribe_id ?? null,
})

function emptyToNull<T extends string | number | null | undefined>(v: T): T | null {
  if (v === '' || v === undefined) return null
  return v
}

function onSubmit() {
  emit('submit', {
    display_name: form.display_name.trim(),
    birth_year: emptyToNull(form.birth_year),
    death_year: emptyToNull(form.death_year),
    region: emptyToNull(form.region?.toString().trim() ?? null),
    biography: emptyToNull(form.biography?.toString().trim() ?? null),
    tribe_id: emptyToNull(form.tribe_id),
  })
}

const inputCls =
  'w-full bg-paper border border-line px-3 py-2 text-sm text-ink focus:border-ochre focus:outline-none'
</script>

<template>
  <form @submit.prevent="onSubmit" class="space-y-6 max-w-2xl">
    <FormField label="display name" required>
      <input v-model="form.display_name" required :class="inputCls" type="text" />
    </FormField>

    <div class="grid grid-cols-2 gap-4">
      <FormField label="birth year">
        <input v-model.number="form.birth_year" :class="inputCls" type="number" />
      </FormField>
      <FormField label="death year">
        <input v-model.number="form.death_year" :class="inputCls" type="number" />
      </FormField>
    </div>

    <FormField label="region">
      <input v-model="form.region" :class="inputCls" type="text" />
    </FormField>

    <FormField label="tribe">
      <select v-model="form.tribe_id" :class="inputCls">
        <option :value="null">-</option>
        <option v-for="t in tribes.items" :key="t.id" :value="t.id">
          {{ t.name }}
        </option>
      </select>
    </FormField>

    <FormField label="biography">
      <textarea v-model="form.biography" :class="inputCls" rows="6" />
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
