import { defineStore } from 'pinia'
import { ref } from 'vue'
import api, { extractError } from '@/api/client'
import type { Artist, ArtistInput } from '@/api/types'

export const useArtistsStore = defineStore('artists', () => {
  const items = ref<Artist[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      const { data } = await api.get<Artist[]>('/artists')
      items.value = data
    } catch (e) {
      error.value = extractError(e)
    } finally {
      loading.value = false
    }
  }

  async function findById(id: string): Promise<Artist> {
    const { data } = await api.get<Artist>(`/artists/${id}`)
    return data
  }

  async function create(input: ArtistInput): Promise<Artist> {
    try {
      const { data } = await api.post<Artist>('/artists', input)
      items.value = [...items.value, data]
      return data
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  async function update(id: string, input: ArtistInput): Promise<Artist> {
    try {
      const { data } = await api.put<Artist>(`/artists/${id}`, input)
      items.value = items.value.map((a) => (a.id === id ? data : a))
      return data
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  async function remove(id: string): Promise<void> {
    try {
      await api.delete(`/artists/${id}`)
      items.value = items.value.filter((a) => a.id !== id)
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  return { items, loading, error, fetchAll, findById, create, update, remove }
})
