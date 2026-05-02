// Helpers for in-textarea interactions: click-to-open URLs (A2) and
// click-to-toggle checklists (A3). All functions return data only —
// they don't touch the DOM or call any Tauri command. The caller wires
// the actual side effects (open URL, persist content) so the same
// helpers can be unit-tested without a runtime.

const URL_RE = /\b((?:https?|file|mailto):[^\s)<>"']+|www\.[^\s)<>"']+)/g;

/**
 * Find the URL surrounding `position` in `text`, if any. Returns the URL
 * string verbatim (so `www.foo.com` stays without the protocol — caller
 * is responsible for normalising before opening).
 */
export function urlAtPosition(text: string, position: number): string | null {
  URL_RE.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = URL_RE.exec(text)) !== null) {
    const start = m.index;
    const end = start + m[0].length;
    if (position >= start && position <= end) {
      return m[0];
    }
  }
  return null;
}

/** Normalise a URL for `open_url`: prepend https:// when the user
 *  typed `www.foo.com` without a scheme. */
export function normaliseUrl(url: string): string {
  if (/^[a-z]+:/i.test(url)) return url;
  return `https://${url}`;
}

const CHECKLIST_RE = /^(\s*[-*]\s)\[( |x|X)\]/;

export interface ChecklistToggleResult {
  content: string;
  /** Caret position to restore after the toggle so the cursor doesn't
   * jump elsewhere. */
  caret: number;
}

/**
 * If `position` falls inside the `[ ]` / `[x]` of a checklist line,
 * return the new content with the bracket flipped, plus the caret
 * position to keep the user's cursor where it was (relative to the
 * unchanged surrounding text). Returns null when the position is not on
 * a checkbox marker.
 */
export function toggleChecklistAt(text: string, position: number): ChecklistToggleResult | null {
  // Resolve the line containing `position`.
  const lineStart = text.lastIndexOf('\n', position - 1) + 1;
  let lineEnd = text.indexOf('\n', position);
  if (lineEnd === -1) lineEnd = text.length;
  const line = text.slice(lineStart, lineEnd);
  const m = line.match(CHECKLIST_RE);
  if (!m) return null;
  const prefix = m[1];
  const bracketStart = lineStart + prefix.length; // points at '['
  const bracketEnd = bracketStart + 3; // points just after ']'
  if (position < bracketStart || position > bracketEnd) return null;
  const next = m[2] === ' ' ? 'x' : ' ';
  const content = text.slice(0, bracketStart) + `[${next}]` + text.slice(bracketStart + 3);
  // Caret stays put — character count of the line is preserved.
  return { content, caret: position };
}
