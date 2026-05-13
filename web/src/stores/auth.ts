import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import api, { extractError } from '@/api/client'
import type { AuthResponse, User } from '@/api/types'

const TOKEN_KEY = 'auth.token'
const USER_KEY = 'auth.user'

interface Persisted {
  token: string | null
  user: User | null
}

// Hydrate the store from localStorage at module-init time. If the stored JSON
// is malformed (manual tampering, schema drift), drop the cached values rather
// than crashing the whole app.
function loadFromStorage(): Persisted {
  try {
    const token = localStorage.getItem(TOKEN_KEY)
    const userRaw = localStorage.getItem(USER_KEY)
    const user = userRaw ? (JSON.parse(userRaw) as User) : null
    return { token, user }
  } catch {
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(USER_KEY)
    return { token: null, user: null }
  }
}

export const useAuthStore = defineStore('auth', () => {
  const persisted = loadFromStorage()
  const token = ref<string | null>(persisted.token)
  const user = ref<User | null>(persisted.user)

  const isAuthenticated = computed(() => !!token.value)
  const isAdmin = computed(() => user.value?.role === 'Admin')

  // Mirror in-memory state to localStorage so refresh keeps the session. We
  // don't try to validate the token here; the next protected request will
  // 401 if it's expired, and the response interceptor clears it then.
  watch(
    [token, user],
    ([t, u]) => {
      if (t) {
        localStorage.setItem(TOKEN_KEY, t)
        localStorage.setItem(USER_KEY, JSON.stringify(u))
      } else {
        localStorage.removeItem(TOKEN_KEY)
        localStorage.removeItem(USER_KEY)
      }
    },
    { deep: true },
  )

  function setSession(res: AuthResponse) {
    token.value = res.token
    user.value = res.user
  }

  async function login(email: string, password: string): Promise<User> {
    try {
      const { data } = await api.post<AuthResponse>('/auth/login', { email, password })
      setSession(data)
      return data.user
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  async function register(email: string, password: string): Promise<User> {
    try {
      const { data } = await api.post<AuthResponse>('/auth/register', { email, password })
      setSession(data)
      return data.user
    } catch (e) {
      throw new Error(extractError(e))
    }
  }

  /// Refresh the cached user from `/auth/me`. Useful after edits to your own
  /// account, or to detect a server-side role change since you logged in.
  async function refreshMe(): Promise<void> {
    try {
      const { data } = await api.get<User>('/auth/me')
      user.value = data
    } catch (e) {
      // 401 will fire the response interceptor and clear us — anything else
      // is non-fatal; keep the cached user.
      if (!isAxiosUnauthorized(e)) throw new Error(extractError(e))
    }
  }

  function logout() {
    token.value = null
    user.value = null
  }

  return {
    token,
    user,
    isAuthenticated,
    isAdmin,
    login,
    register,
    refreshMe,
    logout,
  }
})

function isAxiosUnauthorized(err: unknown): boolean {
  return (
    typeof err === 'object' &&
    err !== null &&
    'response' in err &&
    (err as { response?: { status?: number } }).response?.status === 401
  )
}
