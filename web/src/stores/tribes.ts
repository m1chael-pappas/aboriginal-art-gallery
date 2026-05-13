import { defineStore } from 'pinia'
import { ref } from 'vue'
import api, { extractError } from '@/api/client'
import type { Tribe, TribeInput } from '@/api/types'

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

  async function findById(id: string): Promise<Tribe> {
    const { data } = await api.get<Tribe>(`/tribes/${id}`)
    return data
  }

  async function create(input: TribeInput): Promise<Tribe> {
    try {
      const { data } = await api.post<Tribe>('/tribes', input)
      items.value = [...items.value, data]
      return data
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  async function update(id: string, input: TribeInput): Promise<Tribe> {
    try {
      const { data } = await api.put<Tribe>(`/tribes/${id}`, input)
      items.value = items.value.map((t) => (t.id === id ? data : t))
      return data
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  async function remove(id: string): Promise<void> {
    try {
      await api.delete(`/tribes/${id}`)
      items.value = items.value.filter((t) => t.id !== id)
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  return { items, loading, error, fetchAll, findById, create, update, remove }
})
