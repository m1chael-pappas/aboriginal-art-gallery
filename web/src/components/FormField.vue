<script setup lang="ts">
defineProps<{
  label: string
  required?: boolean
  hint?: string
}>()
</script>

<template>
  <label class="block">
    <span
      class="font-mono text-[10px] tracking-widest uppercase text-muted flex items-center gap-2"
    >
      <span>{{ label }}</span>
      <span v-if="required" class="text-ochre">·</span>
      <span v-if="hint" class="normal-case tracking-normal text-muted/80">
        {{ hint }}
      </span>
    </span>
    <div class="mt-1">
      <slot />
    </div>
  </label>
</template>
