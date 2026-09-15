import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import { AxiosError, AxiosHeaders, type AxiosResponse } from 'axios'
import { createPinia, setActivePinia } from 'pinia'
import api from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import type { AuthResponse, User } from '@/api/types'

const admin: User = {
  id: 'u1',
  email: 'admin@gallery.local',
  role: 'Admin',
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
}

function ok<T>(data: T): AxiosResponse<T> {
  return {
    data,
    status: 200,
    statusText: 'OK',
    headers: {},
    config: { headers: new AxiosHeaders() },
  }
}

describe('auth store', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
  })

  it('login stores the session, exposes the role and persists it to localStorage', async () => {
    const session: AuthResponse = { token: 'jwt-123', user: admin }
    const post = vi.spyOn(api, 'post').mockResolvedValue(ok(session))
    const auth = useAuthStore()

    await auth.login('admin@gallery.local', 'admin-demo-pw')
    await nextTick()

    expect(post).toHaveBeenCalledWith('/auth/login', {
      email: 'admin@gallery.local',
      password: 'admin-demo-pw',
    })
    expect(auth.isAuthenticated).toBe(true)
    expect(auth.isAdmin).toBe(true)
    expect(localStorage.getItem('auth.token')).toBe('jwt-123')
    expect(JSON.parse(localStorage.getItem('auth.user') ?? 'null')).toEqual(admin)
  })

  it('login surfaces the API error message', async () => {
    const response = { ...ok({ error: 'unauthorized: invalid credentials' }), status: 401 }
    vi.spyOn(api, 'post').mockRejectedValue(
      new AxiosError('Request failed', 'ERR_BAD_REQUEST', undefined, undefined, response),
    )
    const auth = useAuthStore()

    await expect(auth.login('admin@gallery.local', 'wrong')).rejects.toThrow(
      'unauthorized: invalid credentials',
    )
    expect(auth.isAuthenticated).toBe(false)
  })

  it('hydrates an existing session from localStorage', () => {
    localStorage.setItem('auth.token', 'jwt-abc')
    localStorage.setItem('auth.user', JSON.stringify({ ...admin, role: 'User' }))

    const auth = useAuthStore()

    expect(auth.isAuthenticated).toBe(true)
    expect(auth.isAdmin).toBe(false)
  })

  it('drops a corrupted persisted user instead of crashing', () => {
    localStorage.setItem('auth.token', 'jwt-abc')
    localStorage.setItem('auth.user', '{not json')

    const auth = useAuthStore()

    expect(auth.isAuthenticated).toBe(false)
    expect(localStorage.getItem('auth.token')).toBeNull()
  })

  it('logout clears memory and storage', async () => {
    localStorage.setItem('auth.token', 'jwt-abc')
    localStorage.setItem('auth.user', JSON.stringify(admin))
    const auth = useAuthStore()

    auth.logout()
    await nextTick()

    expect(auth.isAuthenticated).toBe(false)
    expect(localStorage.getItem('auth.token')).toBeNull()
    expect(localStorage.getItem('auth.user')).toBeNull()
  })
})
