import { defineStore } from 'pinia'
import { ref } from 'vue'
import api, { extractError } from '@/api/client'
import type { Artifact, ArtifactInput } from '@/api/types'

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

  async function findById(id: string): Promise<Artifact> {
    const { data } = await api.get<Artifact>(`/artifacts/${id}`)
    return data
  }

  async function create(input: ArtifactInput): Promise<Artifact> {
    try {
      const { data } = await api.post<Artifact>('/artifacts', input)
      items.value = [...items.value, data]
      return data
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  async function update(id: string, input: ArtifactInput): Promise<Artifact> {
    try {
      const { data } = await api.put<Artifact>(`/artifacts/${id}`, input)
      items.value = items.value.map((a) => (a.id === id ? data : a))
      return data
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  async function remove(id: string): Promise<void> {
    try {
      await api.delete(`/artifacts/${id}`)
      items.value = items.value.filter((a) => a.id !== id)
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  return { items, loading, error, fetchAll, findById, create, update, remove }
})
