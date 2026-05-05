import { defineStore } from 'pinia'
import { ref } from 'vue'
import api, { extractError } from '@/api/client'
import type { Tribe } from '@/api/types'

export const useTribesStore = defineStore('tribes', () => {
  const items = ref<Tribe[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      const { data } = await api.get<Tribe[]>('/tribes')
      items.value = data
    } catch (e) {
      error.value = extractError(e)
    } finally {
      loading.value = false
    }
  }

  return { items, loading, error, fetchAll }
})
