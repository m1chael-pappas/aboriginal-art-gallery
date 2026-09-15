import { defineStore } from 'pinia'
import type { Role, User, UserUpdate } from '@/api/types'
import { useResourceCollection } from './resource'

/**
 * Admin-only store backing the Users management screen. Every `/users`
 * endpoint is gated by the `AdminUser` extractor server-side, so the route
 * using this store also carries `meta.requiresAdmin`.
 */
export const useUsersStore = defineStore('users', () => {
  const { items, loading, error, fetchAll, update, remove } = useResourceCollection<
    User,
    UserUpdate
  >('/users')

  /** Changes one user's role. The API refuses demoting the last admin. */
  function setRole(id: string, role: Role): Promise<User> {
    return update(id, { role })
  }

  return { items, loading, error, fetchAll, setRole, remove }
})
