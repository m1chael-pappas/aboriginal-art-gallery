import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import router from '@/router'
import { useAuthStore } from '@/stores/auth'
import type { User } from '@/api/types'

function signIn(role: User['role']) {
  const auth = useAuthStore()
  auth.token = 'jwt'
  auth.user = {
    id: 'u1',
    email: 'someone@gallery.local',
    role,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  }
}

describe('admin route guard', () => {
  beforeEach(async () => {
    localStorage.clear()
    setActivePinia(createPinia())
    await router.push('/')
  })

  it('sends anonymous visitors to login with a redirect back', async () => {
    await router.push('/users')

    expect(router.currentRoute.value.name).toBe('login')
    expect(router.currentRoute.value.query.redirect).toBe('/users')
  })

  it('sends signed-in non-admins home', async () => {
    signIn('User')

    await router.push('/artists/new')

    expect(router.currentRoute.value.name).toBe('home')
  })

  it('lets admins through', async () => {
    signIn('Admin')

    await router.push('/users')

    expect(router.currentRoute.value.name).toBe('users')
  })

  it('never guards public catalogue routes', async () => {
    await router.push('/artists')

    expect(router.currentRoute.value.name).toBe('artists')
  })
})
