<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import FormField from '@/components/FormField.vue'
import { useArtistsStore } from '@/stores/artists'
import type { ArtifactInput } from '@/api/types'

const props = defineProps<{
  initial: ArtifactInput
  submitLabel: string
  submitting: boolean
  error: string | null
}>()

const emit = defineEmits<{
  submit: [input: ArtifactInput]
  cancel: []
}>()

const artists = useArtistsStore()
onMounted(() => {
  if (artists.items.length === 0) artists.fetchAll()
})

const form = reactive<ArtifactInput>({
  title: props.initial.title,
  artist_id: props.initial.artist_id,
  art_type: props.initial.art_type ?? null,
  art_style: props.initial.art_style ?? null,
  medium: props.initial.medium ?? null,
  year_created: props.initial.year_created ?? null,
  height_cm: props.initial.height_cm ?? null,
  width_cm: props.initial.width_cm ?? null,
  depth_cm: props.initial.depth_cm ?? null,
  description: props.initial.description ?? null,
})

function emptyToNull<T extends string | number | null | undefined>(v: T): T | null {
  if (v === '' || v === undefined) return null
  return v
}

function onSubmit() {
  emit('submit', {
    title: form.title.trim(),
    artist_id: form.artist_id,
    art_type: emptyToNull(form.art_type?.toString().trim() ?? null),
    art_style: emptyToNull(form.art_style?.toString().trim() ?? null),
    medium: emptyToNull(form.medium?.toString().trim() ?? null),
    year_created: emptyToNull(form.year_created),
    height_cm: emptyToNull(form.height_cm),
    width_cm: emptyToNull(form.width_cm),
    depth_cm: emptyToNull(form.depth_cm),
    description: emptyToNull(form.description?.toString().trim() ?? null),
  })
}

const inputCls =
  'w-full bg-paper border border-line px-3 py-2 text-sm text-ink focus:border-ochre focus:outline-none'
</script>

<template>
  <form @submit.prevent="onSubmit" class="space-y-6 max-w-2xl">
    <FormField label="title" required>
      <input v-model="form.title" required :class="inputCls" type="text" />
    </FormField>

    <FormField label="artist" required>
      <select v-model="form.artist_id" required :class="inputCls">
        <option value="" disabled>- select -</option>
        <option v-for="a in artists.items" :key="a.id" :value="a.id">
          {{ a.display_name }}
        </option>
      </select>
    </FormField>

    <div class="grid grid-cols-2 gap-4">
      <FormField label="art type">
        <input v-model="form.art_type" :class="inputCls" type="text" />
      </FormField>
      <FormField label="art style">
        <input v-model="form.art_style" :class="inputCls" type="text" />
      </FormField>
    </div>

    <FormField label="medium" hint="canvas, bark, ochre on board, …">
      <input v-model="form.medium" :class="inputCls" type="text" />
    </FormField>

    <FormField label="year created">
      <input v-model.number="form.year_created" :class="inputCls" type="number" />
    </FormField>

    <div class="grid grid-cols-3 gap-4">
      <FormField label="height (cm)">
        <input v-model.number="form.height_cm" :class="inputCls" type="number" step="0.1" />
      </FormField>
      <FormField label="width (cm)">
        <input v-model.number="form.width_cm" :class="inputCls" type="number" step="0.1" />
      </FormField>
      <FormField label="depth (cm)">
        <input v-model.number="form.depth_cm" :class="inputCls" type="number" step="0.1" />
      </FormField>
    </div>

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
