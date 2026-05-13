<script setup lang="ts">
import { reactive, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import FormField from '@/components/FormField.vue'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

const form = reactive({ email: '', password: '' })
const submitting = ref(false)
const error = ref<string | null>(null)

async function onSubmit() {
  submitting.value = true
  error.value = null
  try {
    await auth.register(form.email.trim(), form.password)
    router.push('/')
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
      auth / register
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Create account</h1>

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

      <FormField label="password" required hint="at least 8 characters">
        <input
          v-model="form.password"
          required
          minlength="8"
          autocomplete="new-password"
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
          {{ submitting ? 'creating…' : 'create account' }}
        </button>
        <RouterLink
          :to="{ name: 'login' }"
          class="font-mono text-xs uppercase tracking-widest text-muted hover:text-ink transition-colors"
        >
          already have one
        </RouterLink>
      </div>
    </form>
  </div>
</template>
