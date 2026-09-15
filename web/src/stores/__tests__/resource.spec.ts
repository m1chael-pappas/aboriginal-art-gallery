import { beforeEach, describe, expect, it, vi } from 'vitest'
import { AxiosError, AxiosHeaders, type AxiosResponse } from 'axios'
import { createPinia, setActivePinia } from 'pinia'
import api from '@/api/client'
import { useArtistsStore } from '@/stores/artists'
import type { Artist, ArtistInput } from '@/api/types'

function artist(overrides: Partial<Artist> = {}): Artist {
  return {
    id: 'a1',
    display_name: 'Emily Kame Kngwarreye',
    birth_year: 1910,
    death_year: 1996,
    region: 'Utopia, Northern Territory',
    biography: null,
    tribe_id: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
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

function apiError(status: number, message: string): AxiosError {
  const response = { ...ok({ error: message }), status }
  return new AxiosError(message, 'ERR_BAD_REQUEST', undefined, undefined, response)
}

describe('resource collection store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('fetchAll loads items from the collection path and clears loading', async () => {
    const get = vi.spyOn(api, 'get').mockResolvedValue(ok([artist()]))
    const store = useArtistsStore()

    const pending = store.fetchAll()
    expect(store.loading).toBe(true)
    await pending

    expect(get).toHaveBeenCalledWith('/artists')
    expect(store.items).toHaveLength(1)
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
  })

  it('fetchAll records the API error message instead of throwing', async () => {
    vi.spyOn(api, 'get').mockRejectedValue(apiError(500, 'internal server error'))
    const store = useArtistsStore()

    await store.fetchAll()

    expect(store.error).toBe('internal server error')
    expect(store.items).toEqual([])
    expect(store.loading).toBe(false)
  })

  it('create posts the input and appends the created row', async () => {
    const created = artist({ id: 'a2', display_name: 'Rover Thomas' })
    const post = vi.spyOn(api, 'post').mockResolvedValue(ok(created))
    const store = useArtistsStore()
    store.items = [artist()]
    const input: ArtistInput = { display_name: 'Rover Thomas' }

    await expect(store.create(input)).resolves.toEqual(created)

    expect(post).toHaveBeenCalledWith('/artists', input)
    expect(store.items.map((a) => a.id)).toEqual(['a1', 'a2'])
  })

  it('update replaces only the matching row', async () => {
    const renamed = artist({ id: 'a1', display_name: 'Emily Kngwarreye' })
    const put = vi.spyOn(api, 'put').mockResolvedValue(ok(renamed))
    const store = useArtistsStore()
    store.items = [artist(), artist({ id: 'a2', display_name: 'Rover Thomas' })]

    await store.update('a1', { display_name: 'Emily Kngwarreye' })

    expect(put).toHaveBeenCalledWith('/artists/a1', { display_name: 'Emily Kngwarreye' })
    expect(store.items.map((a) => a.display_name)).toEqual(['Emily Kngwarreye', 'Rover Thomas'])
  })

  it('remove deletes and drops the row', async () => {
    const del = vi.spyOn(api, 'delete').mockResolvedValue(ok(null))
    const store = useArtistsStore()
    store.items = [artist(), artist({ id: 'a2' })]

    await store.remove('a1')

    expect(del).toHaveBeenCalledWith('/artists/a1')
    expect(store.items.map((a) => a.id)).toEqual(['a2'])
  })

  it('mutations rethrow the API message and leave items untouched', async () => {
    vi.spyOn(api, 'delete').mockRejectedValue(
      apiError(409, 'conflict: cannot delete artist while artifacts reference them'),
    )
    const store = useArtistsStore()
    store.items = [artist()]

    await expect(store.remove('a1')).rejects.toThrow(
      'conflict: cannot delete artist while artifacts reference them',
    )
    expect(store.items).toHaveLength(1)
  })
})
