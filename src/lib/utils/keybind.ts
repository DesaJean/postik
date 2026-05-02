/**
 * Translate a `KeyboardEvent` into the Tauri "CmdOrCtrl+Shift+N" style
 * accelerator string the global-shortcut plugin expects.
 *
 * - Modifier order is fixed: Cmd/Ctrl, Alt, Shift, then the key.
 * - We always use `CmdOrCtrl` rather than the platform-specific
 *   modifier so a binding the user records on macOS still works on
 *   Windows/Linux.
 * - Returns null when the press is "incomplete" (only modifiers, or a
 *   key that doesn't have a useful accelerator name).
 */
export function eventToAccelerator(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (e.metaKey || e.ctrlKey) parts.push('CmdOrCtrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');

  const key = normaliseKey(e.code, e.key);
  if (!key) return null;
  // A bare letter without any modifier would steal regular typing —
  // require at least one modifier for a global shortcut.
  if (parts.length === 0) return null;
  parts.push(key);
  return parts.join('+');
}

function normaliseKey(code: string, key: string): string | null {
  // Letters: code is "KeyA" .. "KeyZ"
  if (code.startsWith('Key') && code.length === 4) {
    return code.slice(3).toUpperCase();
  }
  // Digits: code is "Digit0" .. "Digit9"
  if (code.startsWith('Digit') && code.length === 6) {
    return code.slice(5);
  }
  // Function keys: F1..F24
  if (code.startsWith('F') && /^F\d+$/.test(code)) return code;
  // Arrow keys
  if (code === 'ArrowUp') return 'Up';
  if (code === 'ArrowDown') return 'Down';
  if (code === 'ArrowLeft') return 'Left';
  if (code === 'ArrowRight') return 'Right';
  // Common named keys
  if (code === 'Enter') return 'Enter';
  if (code === 'Space') return 'Space';
  if (code === 'Tab') return 'Tab';
  if (code === 'Backspace') return 'Backspace';
  if (code === 'Escape') return 'Escape';
  if (code === 'Slash') return '/';
  if (code === 'Comma') return ',';
  if (code === 'Period') return '.';
  if (code === 'Semicolon') return ';';
  if (code === 'Quote') return "'";
  if (code === 'Backquote') return '`';
  if (code === 'Minus') return '-';
  if (code === 'Equal') return '=';
  if (code === 'BracketLeft') return '[';
  if (code === 'BracketRight') return ']';
  if (code === 'Backslash') return '\\';
  // Modifier-only presses: ignore (caller treats as "incomplete").
  if (
    code === 'ShiftLeft' ||
    code === 'ShiftRight' ||
    code === 'ControlLeft' ||
    code === 'ControlRight' ||
    code === 'AltLeft' ||
    code === 'AltRight' ||
    code === 'MetaLeft' ||
    code === 'MetaRight'
  ) {
    return null;
  }
  // Fallback: use `key` if it's a single character, otherwise reject.
  if (key && key.length === 1) return key.toUpperCase();
  return null;
}

/** Display an accelerator with platform-appropriate symbols. */
export function prettyAccelerator(accelerator: string, isMac: boolean = isAppleish()): string {
  return accelerator
    .split('+')
    .map((part) => {
      if (part === 'CmdOrCtrl') return isMac ? '⌘' : 'Ctrl';
      if (part === 'Cmd') return isMac ? '⌘' : 'Cmd';
      if (part === 'Ctrl' || part === 'Control') return 'Ctrl';
      if (part === 'Alt') return isMac ? '⌥' : 'Alt';
      if (part === 'Shift') return isMac ? '⇧' : 'Shift';
      return part;
    })
    .join(isMac ? '' : '+');
}

function isAppleish(): boolean {
  if (typeof navigator === 'undefined') return false;
  return /Mac|iPhone|iPad|iPod/.test(navigator.platform || '');
}
