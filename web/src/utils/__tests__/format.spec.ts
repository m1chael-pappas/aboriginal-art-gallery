import { describe, expect, it } from 'vitest'
import {
  blankToNull,
  breadcrumbFor,
  catalogNumber,
  deleteTribePrompt,
  dimensionsLabel,
  lifespan,
} from '@/utils/format'

describe('lifespan', () => {
  it('formats a closed lifespan', () => {
    expect(lifespan(1910, 1996)).toBe('1910-1996')
  })

  it('marks living artists as present', () => {
    expect(lifespan(1945, null)).toBe('1945-present')
  })

  it('marks an unknown birth year', () => {
    expect(lifespan(null, 2003)).toBe('?-2003')
  })

  it('returns a dash when both years are missing, including undefined from optional API fields', () => {
    expect(lifespan(null, null)).toBe('-')
    expect(lifespan(undefined, undefined)).toBe('-')
  })
})

describe('catalogNumber', () => {
  it('pads to three digits with no prefix by default', () => {
    expect(catalogNumber(7)).toBe('007')
  })

  it('applies a prefix and custom width', () => {
    expect(catalogNumber(3, 'T-', 2)).toBe('T-03')
  })

  it('does not truncate numbers wider than the pad width', () => {
    expect(catalogNumber(1234, 'A-')).toBe('A-1234')
  })
})

describe('blankToNull', () => {
  it('trims strings', () => {
    expect(blankToNull('  Utopia  ')).toBe('Utopia')
  })

  it('turns blank strings into null', () => {
    expect(blankToNull('   ')).toBeNull()
    expect(blankToNull('')).toBeNull()
  })

  it('keeps numbers, including zero', () => {
    expect(blankToNull(0)).toBe(0)
    expect(blankToNull(1994)).toBe(1994)
  })

  it('turns null and undefined into null', () => {
    expect(blankToNull(null)).toBeNull()
    expect(blankToNull(undefined)).toBeNull()
  })
})

describe('dimensionsLabel', () => {
  it('shows height and width', () => {
    expect(dimensionsLabel(275, 632, null)).toBe('275 × 632 cm')
  })

  it('adds depth only when it is known', () => {
    expect(dimensionsLabel(40, 30, 5)).toBe('40 × 30 × 5 cm')
  })

  it('marks an unknown height or width', () => {
    expect(dimensionsLabel(undefined, 56, null)).toBe('? × 56 cm')
  })

  it('returns null when nothing is known, including omitted optional fields', () => {
    expect(dimensionsLabel(null, null, null)).toBeNull()
    expect(dimensionsLabel()).toBeNull()
  })
})

describe('breadcrumbFor', () => {
  it('maps home and missing names to index', () => {
    expect(breadcrumbFor('home')).toBe('[ aboriginal_art / index ]')
    expect(breadcrumbFor(undefined)).toBe('[ aboriginal_art / index ]')
  })

  it('turns every dot in a nested route name into a separator', () => {
    expect(breadcrumbFor('artists.detail.edit')).toBe(
      '[ aboriginal_art / artists / detail / edit ]',
    )
  })
})

describe('deleteTribePrompt', () => {
  it('asks plainly when no artist references the tribe', () => {
    expect(deleteTribePrompt('Tiwi', 0)).toBe('Delete "Tiwi"?')
  })

  it('uses the singular for one artist', () => {
    expect(deleteTribePrompt('Tiwi', 1)).toBe('Delete "Tiwi"?\n\n1 artist references this tribe.')
  })

  it('uses the plural for several artists', () => {
    expect(deleteTribePrompt('Yolngu', 3)).toBe(
      'Delete "Yolngu"?\n\n3 artists reference this tribe.',
    )
  })
})
