import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ArtifactForm from '@/components/ArtifactForm.vue'
import { useArtistsStore } from '@/stores/artists'

describe('ArtifactForm', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    const artists = useArtistsStore()
    artists.items = [
      {
        id: 'a1',
        display_name: 'Rover Thomas',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
      },
    ]
    vi.spyOn(artists, 'fetchAll').mockResolvedValue()
  })

  it('emits the chosen artist, numeric dimensions and nulls for blanks', async () => {
    const wrapper = mount(ArtifactForm, {
      props: {
        initial: { title: '', artist_id: '' },
        submitLabel: 'create artifact',
        submitting: false,
        error: null,
      },
    })

    await wrapper.find('input[type="text"]').setValue(' Roads Crossing the Salt Pan ')
    await wrapper.find('select').setValue('a1')
    const [year, height, width, depth] = wrapper.findAll('input[type="number"]')
    await year!.setValue('1986')
    await height!.setValue('90')
    await width!.setValue('180')
    await depth!.setValue('')
    await wrapper.find('form').trigger('submit')

    const [[payload]] = wrapper.emitted('submit') as [[Record<string, unknown>]]
    expect(payload).toMatchObject({
      title: 'Roads Crossing the Salt Pan',
      artist_id: 'a1',
      year_created: 1986,
      height_cm: 90,
      width_cm: 180,
      depth_cm: null,
      art_type: null,
      medium: null,
      description: null,
    })
  })

  it('does not refetch artists that are already loaded', () => {
    const artists = useArtistsStore()
    mount(ArtifactForm, {
      props: {
        initial: { title: 'x', artist_id: 'a1' },
        submitLabel: 'save',
        submitting: false,
        error: null,
      },
    })

    expect(artists.fetchAll).not.toHaveBeenCalled()
  })
})
