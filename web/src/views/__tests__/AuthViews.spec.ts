import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import LoginView from '@/views/LoginView.vue'
import RegisterView from '@/views/RegisterView.vue'
import { useAuthStore } from '@/stores/auth'

const Blank = { template: '<div />' }

function makeRouter(): Router {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'home', component: Blank },
      { path: '/login', name: 'login', component: LoginView },
      { path: '/register', name: 'register', component: RegisterView },
      { path: '/users', name: 'users', component: Blank },
    ],
  })
}

async function mountAt(component: typeof LoginView, path: string) {
  const pinia = createPinia()
  setActivePinia(pinia)
  const router = makeRouter()
  await router.push(path)
  const wrapper = mount(component, { global: { plugins: [pinia, router] } })
  return { wrapper, router }
}

async function fillAndSubmit(wrapper: Awaited<ReturnType<typeof mountAt>>['wrapper']) {
  await wrapper.find('input[type="email"]').setValue('  admin@gallery.local ')
  await wrapper.find('input[type="password"]').setValue('secret-pw')
  await wrapper.find('form').trigger('submit')
  await flushPromises()
}

describe('LoginView', () => {
  beforeEach(() => localStorage.clear())

  it('signs in with a trimmed email and follows the redirect query', async () => {
    const { wrapper, router } = await mountAt(LoginView, '/login?redirect=/users')
    const login = vi.spyOn(useAuthStore(), 'login').mockResolvedValue({} as never)

    await fillAndSubmit(wrapper)

    expect(login).toHaveBeenCalledWith('admin@gallery.local', 'secret-pw')
    expect(router.currentRoute.value.fullPath).toBe('/users')
  })

  it('shows the API error and stays on the page when sign-in fails', async () => {
    const { wrapper, router } = await mountAt(LoginView, '/login')
    vi.spyOn(useAuthStore(), 'login').mockRejectedValue(
      new Error('unauthorized: invalid credentials'),
    )

    await fillAndSubmit(wrapper)

    expect(wrapper.text()).toContain('unauthorized: invalid credentials')
    expect(router.currentRoute.value.name).toBe('login')
    expect(wrapper.find('button[type="submit"]').attributes('disabled')).toBeUndefined()
  })

  it('labels the email and password inputs', async () => {
    const { wrapper } = await mountAt(LoginView, '/login')

    const ids = wrapper.findAll('label').map((l) => l.attributes('for'))
    expect(ids).toHaveLength(2)
    for (const id of ids) expect(wrapper.find(`input[id="${id}"]`).exists()).toBe(true)
  })
})

describe('RegisterView', () => {
  beforeEach(() => localStorage.clear())

  it('registers and goes home', async () => {
    const { wrapper, router } = await mountAt(RegisterView, '/register')
    const register = vi.spyOn(useAuthStore(), 'register').mockResolvedValue({} as never)

    await fillAndSubmit(wrapper)

    expect(register).toHaveBeenCalledWith('admin@gallery.local', 'secret-pw')
    expect(router.currentRoute.value.name).toBe('home')
  })

  it('shows a conflict from the API', async () => {
    const { wrapper } = await mountAt(RegisterView, '/register')
    vi.spyOn(useAuthStore(), 'register').mockRejectedValue(
      new Error('conflict: email already in use'),
    )

    await fillAndSubmit(wrapper)

    expect(wrapper.text()).toContain('conflict: email already in use')
  })
})
