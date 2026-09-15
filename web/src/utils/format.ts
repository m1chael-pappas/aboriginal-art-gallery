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

/**
 * Display dimensions such as `275 × 632 cm` or `38 × ? × 5 cm`. Depth is
 * shown only when known, unknown height or width become `?`, and `null`
 * means there is nothing to show.
 */
export function dimensionsLabel(
  height?: number | null,
  width?: number | null,
  depth?: number | null,
): string | null {
  if (height == null && width == null && depth == null) return null
  const parts: Array<number | null | undefined> = [height, width]
  if (depth != null) parts.push(depth)
  return `${parts.map((d) => d ?? '?').join(' × ')} cm`
}

/** Header breadcrumb for a route name, e.g. `artists.edit` becomes `[ aboriginal_art / artists / edit ]`. */
export function breadcrumbFor(routeName: string | symbol | null | undefined): string {
  const raw = routeName == null || routeName === 'home' ? 'index' : String(routeName)
  return `[ aboriginal_art / ${raw.replaceAll('.', ' / ')} ]`
}

/** Confirmation text before deleting a tribe, warning how many artists still reference it. */
export function deleteTribePrompt(tribeName: string, memberCount: number): string {
  if (memberCount <= 0) return `Delete "${tribeName}"?`
  const references = memberCount === 1 ? 'artist references' : 'artists reference'
  return `Delete "${tribeName}"?\n\n${memberCount} ${references} this tribe.`
}
