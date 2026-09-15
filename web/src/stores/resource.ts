import { ref, type Ref } from 'vue'
import api, { extractError } from '@/api/client'

/**
 * State and REST actions for one API collection such as `/artists`.
 *
 * Every resource store is a Pinia setup store built from this composable, so
 * list loading, error capture and cache updates behave the same across
 * artists, artifacts, tribes and users.
 *
 * `fetchAll` records failures in `error` so index views can render them
 * inline. Mutations throw an `Error` carrying the API's message so forms can
 * show it next to the input, and leave `items` untouched on failure.
 *
 * @param path Collection path relative to the API base URL, e.g. `/artists`.
 */
export function useResourceCollection<T extends { id: string }, Input>(path: string) {
  const items = ref<T[]>([]) as Ref<T[]>
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAll() {
    loading.value = true
    error.value = null
    try {
      const { data } = await api.get<T[]>(path)
      items.value = data
    } catch (e) {
      error.value = extractError(e)
    } finally {
      loading.value = false
    }
  }

  async function findById(id: string): Promise<T> {
    const { data } = await api.get<T>(`${path}/${id}`)
    return data
  }

  async function create(input: Input): Promise<T> {
    const data = await rethrowApiError(api.post<T>(path, input))
    items.value = [...items.value, data]
    return data
  }

  async function update(id: string, input: Partial<Input>): Promise<T> {
    const data = await rethrowApiError(api.put<T>(`${path}/${id}`, input))
    items.value = items.value.map((item) => (item.id === id ? data : item))
    return data
  }

  async function remove(id: string): Promise<void> {
    await rethrowApiError(api.delete(`${path}/${id}`))
    items.value = items.value.filter((item) => item.id !== id)
  }

  return { items, loading, error, fetchAll, findById, create, update, remove }
}

/** Awaits an axios call and rethrows any failure as an `Error` with the API's message. */
async function rethrowApiError<T>(request: Promise<{ data: T }>): Promise<T> {
  try {
    const { data } = await request
    return data
  } catch (e) {
    throw new Error(extractError(e))
  }
}
