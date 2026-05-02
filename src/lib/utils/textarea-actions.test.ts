import { describe, expect, it } from 'vitest';
import { normaliseUrl, toggleChecklistAt, urlAtPosition } from './textarea-actions';

describe('urlAtPosition', () => {
  const text = 'See https://example.com/foo and www.bar.com here.';

  it('finds an https URL at any position inside it', () => {
    expect(urlAtPosition(text, 4)).toBe('https://example.com/foo');
    expect(urlAtPosition(text, 20)).toBe('https://example.com/foo');
  });

  it('finds a www URL when typed without a scheme', () => {
    const idx = text.indexOf('www.');
    expect(urlAtPosition(text, idx + 2)).toBe('www.bar.com');
  });

  it('returns null when the position is on plain text', () => {
    expect(urlAtPosition(text, 0)).toBeNull();
    expect(urlAtPosition(text, text.length - 1)).toBeNull();
  });
});

describe('normaliseUrl', () => {
  it('leaves URLs that already have a scheme untouched', () => {
    expect(normaliseUrl('https://example.com')).toBe('https://example.com');
    expect(normaliseUrl('mailto:nobody@example.com')).toBe('mailto:nobody@example.com');
  });

  it('prepends https:// to bare www URLs', () => {
    expect(normaliseUrl('www.example.com')).toBe('https://www.example.com');
  });
});

describe('toggleChecklistAt', () => {
  it('flips an unchecked box to checked when clicked on the brackets', () => {
    const text = '- [ ] buy milk';
    // Click on the space between brackets.
    const r = toggleChecklistAt(text, 3);
    expect(r?.content).toBe('- [x] buy milk');
  });

  it('flips a checked box to unchecked', () => {
    const text = '- [x] done';
    const r = toggleChecklistAt(text, 3);
    expect(r?.content).toBe('- [ ] done');
  });

  it('respects indentation', () => {
    const text = '  - [ ] sub item';
    const r = toggleChecklistAt(text, 5);
    expect(r?.content).toBe('  - [x] sub item');
  });

  it('handles the * bullet variant', () => {
    const text = '* [ ] alt syntax';
    const r = toggleChecklistAt(text, 3);
    expect(r?.content).toBe('* [x] alt syntax');
  });

  it('returns null when click is outside the brackets', () => {
    const text = '- [ ] buy milk';
    expect(toggleChecklistAt(text, 0)).toBeNull(); // on the dash
    expect(toggleChecklistAt(text, 10)).toBeNull(); // on the word
  });

  it('returns null on a non-checkbox line', () => {
    const text = 'just a regular note';
    expect(toggleChecklistAt(text, 5)).toBeNull();
  });

  it('toggles the right line in a multi-line note', () => {
    const text = 'header\n- [ ] one\n- [ ] two';
    // Click between brackets of the SECOND task.
    const idx = text.lastIndexOf('[ ]') + 1;
    const r = toggleChecklistAt(text, idx);
    expect(r?.content).toBe('header\n- [ ] one\n- [x] two');
  });
});
