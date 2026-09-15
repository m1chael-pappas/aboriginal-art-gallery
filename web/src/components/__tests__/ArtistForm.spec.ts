import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ArtistForm from '@/components/ArtistForm.vue'
import { useTribesStore } from '@/stores/tribes'
import type { ArtistInput } from '@/api/types'

function mountForm(initial: ArtistInput = { display_name: '' }) {
  return mount(ArtistForm, {
    props: { initial, submitLabel: 'create artist', submitting: false, error: null },
  })
}

describe('ArtistForm', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    const tribes = useTribesStore()
    tribes.items = [
      {
        id: 't1',
        name: 'Anmatyerre',
        territory: {},
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
      },
    ]
    vi.spyOn(tribes, 'fetchAll').mockResolvedValue()
  })

  it('emits a trimmed payload with blank optional fields as null', async () => {
    const wrapper = mountForm()

    await wrapper.find('input[type="text"]').setValue('  Rover Thomas  ')
    const [birth, death] = wrapper.findAll('input[type="number"]')
    await birth!.setValue('1926')
    await death!.setValue('')
    await wrapper.find('select').setValue('t1')
    await wrapper.find('textarea').setValue('   ')
    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('submit')).toEqual([
      [
        {
          display_name: 'Rover Thomas',
          birth_year: 1926,
          death_year: null,
          region: null,
          biography: null,
          tribe_id: 't1',
        },
      ],
    ])
  })

  it('pre-fills from the initial value and lists tribes', () => {
    const wrapper = mountForm({
      display_name: 'Emily Kame Kngwarreye',
      birth_year: 1910,
      tribe_id: 't1',
    })

    expect((wrapper.find('input[type="text"]').element as HTMLInputElement).value).toBe(
      'Emily Kame Kngwarreye',
    )
    expect(wrapper.findAll('option').map((o) => o.text())).toEqual(['-', 'Anmatyerre'])
  })

  it('shows the server error and disables submit while saving', () => {
    const wrapper = mount(ArtistForm, {
      props: {
        initial: { display_name: 'x' },
        submitLabel: 'save',
        submitting: true,
        error: 'validation failed: death_year must be greater than or equal to birth_year',
      },
    })

    expect(wrapper.text()).toContain('death_year must be greater than or equal to birth_year')
    const submit = wrapper.find('button[type="submit"]')
    expect(submit.attributes('disabled')).toBeDefined()
    expect(submit.text()).toBe('saving…')
  })

  it('links every label to its control so screen readers announce the field', () => {
    const wrapper = mountForm()

    const labels = wrapper.findAll('label')
    expect(labels.length).toBeGreaterThan(0)
    for (const label of labels) {
      const target = label.attributes('for')
      expect(target, `label "${label.text()}" has no for`).toBeTruthy()
      expect(wrapper.find(`[id="${target}"]`).exists(), `no control for "${label.text()}"`).toBe(true)
    }
  })

  it('emits cancel', async () => {
    const wrapper = mountForm()

    await wrapper.find('button[type="button"]').trigger('click')

    expect(wrapper.emitted('cancel')).toHaveLength(1)
  })
})
