import { defineStore } from 'pinia'
import { ref } from 'vue'
import api, { extractError } from '@/api/client'
import type { Artifact } from '@/api/types'

export const useArtifactsStore = defineStore('artifacts', () => {
  const items = ref<Artifact[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      const { data } = await api.get<Artifact[]>('/artifacts')
      items.value = data
    } catch (e) {
      error.value = extractError(e)
    } finally {
      loading.value = false
    }
  }

  return { items, loading, error, fetchAll }
})
