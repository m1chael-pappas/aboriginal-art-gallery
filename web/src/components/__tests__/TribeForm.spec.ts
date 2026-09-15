import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import TribeForm from '@/components/TribeForm.vue'

describe('TribeForm', () => {
  it('emits a trimmed payload and nulls blank optional fields', async () => {
    const wrapper = mount(TribeForm, {
      props: { initial: { name: '' }, submitLabel: 'create tribe', submitting: false, error: null },
    })

    const [name, region, language] = wrapper.findAll('input')
    await name!.setValue('  Pintupi ')
    await region!.setValue('Western Desert')
    await language!.setValue('   ')
    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('submit')).toEqual([
      [{ name: 'Pintupi', region: 'Western Desert', language_group: null, description: null }],
    ])
  })

  it('pre-fills from the initial tribe and labels every control', () => {
    const wrapper = mount(TribeForm, {
      props: {
        initial: { name: 'Tiwi', region: 'Tiwi Islands', description: 'Island people' },
        submitLabel: 'save',
        submitting: false,
        error: null,
      },
    })

    expect((wrapper.find('input').element as HTMLInputElement).value).toBe('Tiwi')
    expect((wrapper.find('textarea').element as HTMLTextAreaElement).value).toBe('Island people')
    for (const label of wrapper.findAll('label')) {
      expect(wrapper.find(`[id="${label.attributes('for')}"]`).exists()).toBe(true)
    }
  })
})
