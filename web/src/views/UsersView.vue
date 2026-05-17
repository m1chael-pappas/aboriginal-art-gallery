<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useUsersStore } from '@/stores/users'
import { useAuthStore } from '@/stores/auth'
import type { Role, User } from '@/api/types'

const users = useUsersStore()
const auth = useAuthStore()

// Per-row action error + in-flight id, so one failing row doesn't blank the
// whole table and a click can't be double-fired while the request is open.
const rowError = ref<string | null>(null)
const busyId = ref<string | null>(null)

onMounted(() => {
  users.fetchAll()
})

const selfId = computed(() => auth.user?.id ?? null)

function pad(n: number): string {
  return String(n).padStart(3, '0')
}

function shortDate(iso: string): string {
  return iso.slice(0, 10)
}

async function onToggleRole(u: User) {
  const next: Role = u.role === 'Admin' ? 'User' : 'Admin'
  const verb = next === 'Admin' ? 'Promote' : 'Demote'
  if (!window.confirm(`${verb} ${u.email} to ${next}?`)) return
  rowError.value = null
  busyId.value = u.id
  try {
    await users.setRole(u.id, next)
  } catch (e) {
    rowError.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    busyId.value = null
  }
}

async function onDelete(u: User) {
  if (!window.confirm(`Delete ${u.email}? This cannot be undone.`)) return
  rowError.value = null
  busyId.value = u.id
  try {
    await users.remove(u.id)
  } catch (e) {
    rowError.value = e instanceof Error ? e.message : 'unknown error'
  } finally {
    busyId.value = null
  }
}
</script>

<template>
  <div>
    <div class="flex items-baseline justify-between mb-3">
      <div class="font-mono text-[10px] tracking-widest uppercase text-muted">
        index · {{ users.items.length }} {{ users.items.length === 1 ? 'account' : 'accounts' }}
      </div>
      <span class="font-mono text-[10px] uppercase tracking-widest text-ochre">
        admin only
      </span>
    </div>
    <h1 class="text-3xl font-medium text-ink mb-8">Users</h1>

    <div v-if="users.loading" class="text-sm text-muted">Loading…</div>

    <div
      v-else-if="users.error"
      class="p-4 border border-ochre/40 bg-ochre/5 text-sm text-ink"
    >
      {{ users.error }}
    </div>

    <div v-else-if="users.items.length === 0" class="text-sm text-muted">
      No accounts yet.
    </div>

    <div v-else>
      <div
        v-if="rowError"
        class="p-3 mb-4 border border-ochre/40 bg-ochre/5 text-sm text-ink"
      >
        {{ rowError }}
      </div>

      <div
        class="grid grid-cols-[50px_2fr_90px_110px_150px] gap-3 pb-2 border-b border-line text-[10px] font-mono uppercase tracking-wider text-muted"
      >
        <span>#</span>
        <span>email</span>
        <span>role</span>
        <span>joined</span>
        <span class="text-right">actions</span>
      </div>

      <div
        v-for="(u, i) in users.items"
        :key="u.id"
        class="grid grid-cols-[50px_2fr_90px_110px_150px] gap-3 py-3 border-b border-dashed border-line items-center text-sm"
      >
        <span class="font-mono text-xs text-muted">{{ pad(i + 1) }}</span>
        <span class="text-ink">
          {{ u.email }}
          <span
            v-if="u.id === selfId"
            class="font-mono text-[10px] uppercase tracking-widest text-muted"
          >
            · you
          </span>
        </span>
        <span
          class="font-mono text-xs uppercase tracking-widest"
          :class="u.role === 'Admin' ? 'text-ochre' : 'text-muted'"
        >
          {{ u.role }}
        </span>
        <span class="font-mono text-xs text-muted">{{ shortDate(u.created_at) }}</span>

        <span
          class="flex items-center justify-end gap-4 font-mono text-xs uppercase tracking-widest"
        >
          <!-- Role changes on your own account are disallowed in the UI: an
               admin demoting themselves would lose access mid-session. -->
          <button
            type="button"
            :disabled="busyId === u.id || u.id === selfId"
            @click="onToggleRole(u)"
            class="text-ink hover:text-ochre disabled:opacity-30 disabled:hover:text-ink transition-colors"
          >
            {{ u.role === 'Admin' ? 'demote' : 'promote' }}
          </button>
          <!-- The backend also blocks self-delete (409); we hide it for a
               cleaner row rather than show a button that always errors. -->
          <button
            type="button"
            :disabled="busyId === u.id || u.id === selfId"
            @click="onDelete(u)"
            class="text-muted hover:text-ochre disabled:opacity-30 disabled:hover:text-muted transition-colors"
          >
            delete
          </button>
        </span>
      </div>
    </div>
  </div>
</template>
