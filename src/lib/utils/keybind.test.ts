import { describe, expect, it } from 'vitest';
import { eventToAccelerator, prettyAccelerator } from './keybind';

// Plain object cast to KeyboardEvent — Vitest runs in node by default
// and `KeyboardEvent` isn't a global there. The function under test only
// reads a handful of properties, so a literal works fine.
function ke(opts: {
  code?: string;
  key?: string;
  metaKey?: boolean;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
}): KeyboardEvent {
  return {
    code: opts.code ?? '',
    key: opts.key ?? '',
    metaKey: opts.metaKey ?? false,
    ctrlKey: opts.ctrlKey ?? false,
    shiftKey: opts.shiftKey ?? false,
    altKey: opts.altKey ?? false,
  } as KeyboardEvent;
}

describe('eventToAccelerator', () => {
  it('builds CmdOrCtrl+Shift+N', () => {
    const e = ke({ code: 'KeyN', metaKey: true, shiftKey: true });
    expect(eventToAccelerator(e)).toBe('CmdOrCtrl+Shift+N');
  });

  it('treats Ctrl as CmdOrCtrl', () => {
    const e = ke({ code: 'KeyP', ctrlKey: true, shiftKey: true });
    expect(eventToAccelerator(e)).toBe('CmdOrCtrl+Shift+P');
  });

  it('preserves modifier order Cmd → Alt → Shift → key', () => {
    const e = ke({ code: 'KeyZ', metaKey: true, altKey: true, shiftKey: true });
    expect(eventToAccelerator(e)).toBe('CmdOrCtrl+Alt+Shift+Z');
  });

  it('returns null for a bare letter (no modifier)', () => {
    const e = ke({ code: 'KeyA' });
    expect(eventToAccelerator(e)).toBeNull();
  });

  it('returns null when only a modifier is held', () => {
    const e = ke({ code: 'ShiftLeft', shiftKey: true });
    expect(eventToAccelerator(e)).toBeNull();
  });

  it('handles function keys', () => {
    const e = ke({ code: 'F5', metaKey: true });
    expect(eventToAccelerator(e)).toBe('CmdOrCtrl+F5');
  });

  it('handles arrows', () => {
    const e = ke({ code: 'ArrowUp', metaKey: true, shiftKey: true });
    expect(eventToAccelerator(e)).toBe('CmdOrCtrl+Shift+Up');
  });
});

describe('prettyAccelerator', () => {
  it('renders mac symbols', () => {
    expect(prettyAccelerator('CmdOrCtrl+Shift+N', true)).toBe('⌘⇧N');
  });

  it('renders verbose words on non-mac', () => {
    expect(prettyAccelerator('CmdOrCtrl+Shift+N', false)).toBe('Ctrl+Shift+N');
  });

  it('preserves arrow names', () => {
    expect(prettyAccelerator('CmdOrCtrl+Shift+Up', true)).toBe('⌘⇧Up');
  });
});
