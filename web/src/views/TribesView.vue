<script setup lang="ts">
import { onMounted } from 'vue'
import { useTribesStore } from '@/stores/tribes'

const store = useTribesStore()
onMounted(() => store.fetchAll())
</script>

<template>
  <div>
    <div class="flex items-baseline justify-between mb-6">
      <h1 class="text-2xl font-semibold text-stone-900">Tribes</h1>
      <span class="text-sm text-stone-500">
        {{ store.items.length }} {{ store.items.length === 1 ? 'tribe' : 'tribes' }}
      </span>
    </div>

    <div v-if="store.loading" class="text-stone-500">Loading…</div>

    <div
      v-else-if="store.error"
      class="p-4 border border-red-200 bg-red-50 rounded-lg text-red-800"
    >
      {{ store.error }}
    </div>

    <div v-else-if="store.items.length === 0" class="text-stone-500">
      No tribes yet. Create one via Insomnia.
    </div>

    <ul v-else class="grid gap-3">
      <li
        v-for="tribe in store.items"
        :key="tribe.id"
        class="p-4 border border-stone-200 rounded-lg bg-white"
      >
        <h3 class="font-semibold text-stone-900">{{ tribe.name }}</h3>
        <p class="text-sm text-stone-600 mt-1">
          <span v-if="tribe.region">{{ tribe.region }}</span>
          <span v-if="tribe.region && tribe.language_group"> · </span>
          <span v-if="tribe.language_group">{{ tribe.language_group }}</span>
        </p>
        <p
          v-if="tribe.description"
          class="text-sm text-stone-500 mt-2 line-clamp-2"
        >
          {{ tribe.description }}
        </p>
      </li>
    </ul>
  </div>
</template>
