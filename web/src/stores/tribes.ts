import { defineStore } from 'pinia'
import type { Tribe, TribeInput } from '@/api/types'
import { useResourceCollection } from './resource'

/** Public catalogue of tribes. Writes require the Admin role server-side. */
export const useTribesStore = defineStore('tribes', () =>
  useResourceCollection<Tribe, TribeInput>('/tribes'),
)
