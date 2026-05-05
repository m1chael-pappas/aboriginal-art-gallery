import axios from 'axios'

const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL ?? 'http://localhost:8080',
  headers: {
    'Content-Type': 'application/json',
  },
})

export default api

// Pull a human-readable message out of any thrown value. Backend errors
// arrive as { error: string } and we want to surface that text rather
// than the generic axios message.
export function extractError(err: unknown): string {
  if (axios.isAxiosError(err)) {
    const body = err.response?.data as { error?: string } | undefined
    if (body?.error) return body.error
    return err.message
  }
  return err instanceof Error ? err.message : 'unknown error'
}
