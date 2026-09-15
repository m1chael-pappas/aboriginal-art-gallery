import { defineStore } from 'pinia'
import type { Artist, ArtistInput } from '@/api/types'
import { useResourceCollection } from './resource'

/** Public catalogue of artists. Writes require the Admin role server-side. */
export const useArtistsStore = defineStore('artists', () =>
  useResourceCollection<Artist, ArtistInput>('/artists'),
)
