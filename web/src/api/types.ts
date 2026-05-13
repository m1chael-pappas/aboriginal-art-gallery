// Hand-written types mirroring the Rust models in api/src/{artists,artifacts,tribes,users}/model.rs.
// Will be replaced by typeshare-generated output in a later commit.

export interface Artist {
  id: string
  display_name: string
  birth_year: number | null
  death_year: number | null
  region: string | null
  biography: string | null
  tribe_id: string | null
  created_at: string
  updated_at: string
}

export interface ArtistInput {
  display_name: string
  birth_year?: number | null
  death_year?: number | null
  region?: string | null
  biography?: string | null
  tribe_id?: string | null
}

export interface Artifact {
  id: string
  title: string
  artist_id: string
  art_type: string | null
  art_style: string | null
  medium: string | null
  year_created: number | null
  height_cm: number | null
  width_cm: number | null
  depth_cm: number | null
  description: string | null
  created_at: string
  updated_at: string
}

export interface ArtifactInput {
  title: string
  artist_id: string
  art_type?: string | null
  art_style?: string | null
  medium?: string | null
  year_created?: number | null
  height_cm?: number | null
  width_cm?: number | null
  depth_cm?: number | null
  description?: string | null
}

export interface Tribe {
  id: string
  name: string
  region: string | null
  language_group: string | null
  description: string | null
  created_at: string
  updated_at: string
}

export interface TribeInput {
  name: string
  region?: string | null
  language_group?: string | null
  description?: string | null
}

export type Role = 'User' | 'Admin'

export interface User {
  id: string
  email: string
  role: Role
  created_at: string
  updated_at: string
}

export interface RegisterInput {
  email: string
  password: string
}

export interface LoginInput {
  email: string
  password: string
}

export interface AuthResponse {
  token: string
  user: User
}
