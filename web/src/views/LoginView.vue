<script setup lang="ts">
import { reactive, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import FormField from '@/components/FormField.vue'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

const form = reactive({ email: '', password: '' })
const submitting = ref(false)
const error = ref<string | null>(null)

async function onSubmit() {
  submitting.value = true
  error.value = null
  try {
    await auth.login(form.email.trim(), form.password)
    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/'
    router.push(redirect)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    submitting.value = false
  }
}

const inputCls =
  'w-full bg-paper border border-line px-3 py-2 text-sm text-ink focus:border-ochre focus:outline-none'
</script>

<template>
  <div>
    <div class="font-mono text-[10px] tracking-widest uppercase text-muted mb-3">
      auth / sign in
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Sign in</h1>

    <form @submit.prevent="onSubmit" class="space-y-6 max-w-md">
      <FormField label="email" required>
        <input
          v-model="form.email"
          required
          autocomplete="email"
          :class="inputCls"
          type="email"
        />
      </FormField>

      <FormField label="password" required>
        <input
          v-model="form.password"
          required
          autocomplete="current-password"
          :class="inputCls"
          type="password"
        />
      </FormField>

      <div v-if="error" class="p-3 border border-ochre/40 bg-ochre/5 text-sm text-ink">
        {{ error }}
      </div>

      <div class="flex items-center gap-4 pt-2 border-t border-dashed border-line">
        <button
          type="submit"
          :disabled="submitting"
          class="font-mono text-xs uppercase tracking-widest text-ink hover:text-ochre disabled:opacity-50 transition-colors"
        >
          {{ submitting ? 'signing in…' : 'sign in' }}
        </button>
        <RouterLink
          :to="{ name: 'register' }"
          class="font-mono text-xs uppercase tracking-widest text-muted hover:text-ink transition-colors"
        >
          create account
        </RouterLink>
      </div>
    </form>
  </div>
</template>
