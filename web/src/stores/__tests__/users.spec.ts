import { beforeEach, describe, expect, it, vi } from 'vitest'
import { AxiosHeaders, type AxiosResponse } from 'axios'
import { createPinia, setActivePinia } from 'pinia'
import api from '@/api/client'
import { useUsersStore } from '@/stores/users'
import type { User } from '@/api/types'

function user(overrides: Partial<User> = {}): User {
  return {
    id: 'u1',
    email: 'curator@gallery.local',
    role: 'User',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

describe('users store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('setRole sends only the role and updates the row', async () => {
    const promoted = user({ role: 'Admin' })
    const response: AxiosResponse<User> = {
      data: promoted,
      status: 200,
      statusText: 'OK',
      headers: {},
      config: { headers: new AxiosHeaders() },
    }
    const put = vi.spyOn(api, 'put').mockResolvedValue(response)
    const users = useUsersStore()
    users.items = [user(), user({ id: 'u2', email: 'other@gallery.local' })]

    await expect(users.setRole('u1', 'Admin')).resolves.toEqual(promoted)

    expect(put).toHaveBeenCalledWith('/users/u1', { role: 'Admin' })
    expect(users.items.map((u) => u.role)).toEqual(['Admin', 'User'])
  })
})
