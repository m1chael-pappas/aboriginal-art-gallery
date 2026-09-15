import { beforeEach, describe, expect, it } from 'vitest'
import { nextTick } from 'vue'
import { AxiosError, AxiosHeaders, type InternalAxiosRequestConfig } from 'axios'
import { createPinia, setActivePinia } from 'pinia'
import api, { extractError } from '@/api/client'
import { useAuthStore } from '@/stores/auth'

/** Replaces the network with a stub that records the request and replies with `status`. */
function stubAdapter(status: number, body: unknown) {
  const seen: InternalAxiosRequestConfig[] = []
  api.defaults.adapter = async (config) => {
    seen.push(config)
    const response = { data: body, status, statusText: '', headers: {}, config }
    if (status >= 400) {
      throw new AxiosError('Request failed', 'ERR_BAD_REQUEST', config, undefined, response)
    }
    return response
  }
  return seen
}

describe('extractError', () => {
  it('prefers the API error body', () => {
    const response = {
      data: { error: 'validation failed: title cannot be empty' },
      status: 400,
      statusText: '',
      headers: {},
      config: { headers: new AxiosHeaders() },
    }
    const err = new AxiosError(
      'Request failed with status code 400',
      'ERR_BAD_REQUEST',
      undefined,
      undefined,
      response,
    )
    expect(extractError(err)).toBe('validation failed: title cannot be empty')
  })

  it('falls back to the axios message when there is no body', () => {
    expect(extractError(new AxiosError('Network Error'))).toBe('Network Error')
  })

  it('handles plain errors and unknown values', () => {
    expect(extractError(new Error('boom'))).toBe('boom')
    expect(extractError('nope')).toBe('unknown error')
  })
})

describe('api client interceptors', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
  })

  it('attaches the bearer token when signed in', async () => {
    useAuthStore().token = 'jwt-xyz'
    const seen = stubAdapter(200, [])

    await api.get('/users')

    expect(seen[0]?.headers.Authorization).toBe('Bearer jwt-xyz')
  })

  it('sends no Authorization header when signed out', async () => {
    const seen = stubAdapter(200, [])

    await api.get('/artists')

    expect(seen[0]?.headers.Authorization).toBeUndefined()
  })

  it('clears the session when an authenticated call returns 401', async () => {
    window.history.replaceState({}, '', '/login')
    const auth = useAuthStore()
    auth.token = 'expired'
    stubAdapter(401, { error: 'unauthorized: token expired' })

    await expect(api.get('/auth/me')).rejects.toBeInstanceOf(AxiosError)
    await nextTick()

    expect(auth.token).toBeNull()
  })

  it('keeps the session when the failing call is the login itself', async () => {
    const auth = useAuthStore()
    auth.token = 'still-valid'
    stubAdapter(401, { error: 'unauthorized: invalid credentials' })

    await expect(api.post('/auth/login', {})).rejects.toBeInstanceOf(AxiosError)

    expect(auth.token).toBe('still-valid')
  })
})
