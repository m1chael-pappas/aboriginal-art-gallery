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

// Lazy import — the auth store and this module form a cycle (store uses api,
// api needs the store for interceptors), and resolving the import inside the
// interceptor callbacks avoids the circular-init footgun.
async function getAuthStore() {
  const { useAuthStore } = await import('@/stores/auth')
  return useAuthStore()
}

// Attach Authorization to every outbound request when we have a token.
api.interceptors.request.use(async (config) => {
  const auth = await getAuthStore()
  if (auth.token) {
    config.headers.Authorization = `Bearer ${auth.token}`
  }
  return config
})

// Globally handle 401: clear the session and bounce to /login. Skipped when
// the user is already on /login (would create a redirect loop on bad creds)
// and when the failing request was itself /auth/login (login forms surface
// the error inline rather than reloading the page).
api.interceptors.response.use(
  (res) => res,
  async (err) => {
    if (axios.isAxiosError(err) && err.response?.status === 401) {
      const requestUrl = err.config?.url ?? ''
      const isLoginAttempt = requestUrl.includes('/auth/login')
      const alreadyOnLogin = window.location.pathname.startsWith('/login')

      if (!isLoginAttempt) {
        const auth = await getAuthStore()
        auth.logout()
        if (!alreadyOnLogin) {
          window.location.href = '/login'
        }
      }
    }
    return Promise.reject(err)
  },
)
