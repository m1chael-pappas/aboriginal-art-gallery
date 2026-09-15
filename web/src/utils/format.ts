/**
 * Display lifespan such as `1910-1996`, `1945-present` or `?-2003`.
 * Returns `-` when both years are unknown. Accepts `undefined` because the
 * generated API types mark nullable columns as optional.
 */
export function lifespan(birth?: number | null, death?: number | null): string {
  if (birth == null && death == null) return '-'
  return `${birth ?? '?'}-${death ?? 'present'}`
}

/** Zero-padded catalogue number for list rows, e.g. `A-007` or `T-03`. */
export function catalogNumber(n: number, prefix = '', width = 3): string {
  return `${prefix}${String(n).padStart(width, '0')}`
}

/**
 * Normalises an optional form value before it is sent to the API. Strings
 * are trimmed and blank ones become `null`. `v-model.number` yields `''` for
 * an empty input, which also becomes `null`.
 */
export function blankToNull<T extends string | number>(value: T | '' | null | undefined): T | null {
  if (value == null) return null
  if (typeof value === 'string') {
    const trimmed = value.trim()
    return trimmed === '' ? null : (trimmed as T)
  }
  return value
}
