import { marked } from 'marked';
import DOMPurify from 'dompurify';

// Configure marked once. GFM enabled gives us task-list checkboxes
// (`- [ ]` → <input type="checkbox" disabled>), strikethroughs,
// auto-linking, and tables. We keep `breaks: true` so single newlines
// render as <br>, matching how a sticky note "feels" when typed.
marked.setOptions({
  gfm: true,
  breaks: true,
});

/**
 * Render markdown to safe HTML. Sanitised with DOMPurify; the default
 * config already strips <script>, on*= handlers, and javascript: URLs.
 * We additionally allow `data-line` so click handlers can find the
 * source line for an interaction (e.g. checkbox toggling).
 */
export function renderMarkdown(source: string): string {
  // marked.parse returns a string in sync mode (the default).
  const raw = marked.parse(source, { async: false }) as string;
  return DOMPurify.sanitize(raw, {
    ADD_ATTR: ['target', 'rel'],
  });
}
