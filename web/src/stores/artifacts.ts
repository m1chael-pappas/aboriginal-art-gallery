import { defineStore } from 'pinia'
import type { Artifact, ArtifactInput } from '@/api/types'
import { useResourceCollection } from './resource'

/** Public catalogue of artifacts. Writes require the Admin role server-side. */
export const useArtifactsStore = defineStore('artifacts', () =>
  useResourceCollection<Artifact, ArtifactInput>('/artifacts'),
)
