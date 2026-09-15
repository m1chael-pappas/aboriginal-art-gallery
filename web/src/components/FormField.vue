<script setup lang="ts">
import { useId } from 'vue'

defineProps<{
  label: string
  required?: boolean
  hint?: string
}>()

defineSlots<{
  /** The form control. Bind the provided `id` so the label points at it. */
  default(props: { id: string }): unknown
}>()

const id = useId()
</script>

<template>
  <div class="block">
    <label
      :for="id"
      class="font-mono text-[10px] tracking-widest uppercase text-muted flex items-center gap-2"
    >
      <span>{{ label }}</span>
      <span v-if="required" class="text-ochre" aria-hidden="true">·</span>
      <span v-if="hint" class="normal-case tracking-normal text-muted/80">
        {{ hint }}
      </span>
    </label>
    <div class="mt-1">
      <slot :id="id" />
    </div>
  </div>
</template>
