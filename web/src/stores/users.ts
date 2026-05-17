import { defineStore } from 'pinia'
import { ref } from 'vue'
import api, { extractError } from '@/api/client'
import type { Role, User } from '@/api/types'

// Admin-only store backing the Users management screen. Every endpoint here
// is gated behind the Admin role server-side (`AdminUser` extractor), so the
// route that uses this store also carries `meta.requiresAdmin`.
export const useUsersStore = defineStore('users', () => {
  const items = ref<User[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // GET /users — keeps the failure message in `error` (non-throwing) so the
  // list view can render it inline, matching the other index views.
  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      const { data } = await api.get<User[]>('/users')
      items.value = data
    } catch (e) {
      error.value = extractError(e)
    } finally {
      loading.value = false
    }
  }

  // PUT /users/{id} with just the role field. Mutating actions throw so the
  // caller can surface a per-row error and leave the list intact.
  async function setRole(id: string, role: Role): Promise<User> {
    try {
      const { data } = await api.put<User>(`/users/${id}`, { role })
      items.value = items.value.map((u) => (u.id === id ? data : u))
      return data
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  // DELETE /users/{id}. The backend refuses an admin deleting their own
  // account (409); the view also hides the control on the self row.
  async function remove(id: string): Promise<void> {
    try {
      await api.delete(`/users/${id}`)
      items.value = items.value.filter((u) => u.id !== id)
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  return { items, loading, error, fetchAll, setRole, remove }
})
