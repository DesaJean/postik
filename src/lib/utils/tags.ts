/**
 * Tag-list helpers. Tags travel as a comma-separated lowercase string
 * (e.g. `work,1on1,urgent`); the UI splits them into chips and joins
 * back when persisting.
 */

export function parseTags(raw: string | null | undefined): string[] {
  if (!raw) return [];
  return Array.from(
    new Set(
      raw
        .split(',')
        .map((t) => t.trim().toLowerCase())
        .filter((t) => t.length > 0),
    ),
  );
}

export function joinTags(tags: string[]): string {
  return parseTags(tags.join(',')).join(',');
}

/** Distinct tags across a list of notes, sorted alphabetically. */
export function distinctTags(notes: Array<{ tags: string | null }>): string[] {
  const set = new Set<string>();
  for (const n of notes) {
    for (const t of parseTags(n.tags)) set.add(t);
  }
  return Array.from(set).sort();
}
