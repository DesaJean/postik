import type { TimerMode } from '../types';

export interface ParsedTimer {
  mode: TimerMode;
  durationSeconds: number | null;
}

const COUNTDOWN_RE = /^(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?$/;

/**
 * Parse a user-typed timer string into a normalized command.
 *
 * Accepts:
 *   "25m"        → countdown 1500s
 *   "1h30m"      → countdown 5400s
 *   "90s"        → countdown 90s
 *   "2h"         → countdown 7200s
 *   "pomo"       → pomodoro
 *   "stopwatch"  → stopwatch
 *
 * Returns null for unparseable input. An empty string is treated as stopwatch.
 */
export function parseTimerInput(raw: string): ParsedTimer | null {
  const input = raw.trim().toLowerCase();
  if (input === '' || input === 'stopwatch' || input === 'sw') {
    return { mode: 'stopwatch', durationSeconds: null };
  }
  if (input === 'pomo' || input === 'pomodoro') {
    return { mode: 'pomodoro', durationSeconds: 25 * 60 };
  }

  const match = COUNTDOWN_RE.exec(input);
  if (!match) return null;
  const [, h, m, s] = match;
  if (!h && !m && !s) return null;

  const hours = h ? parseInt(h, 10) : 0;
  const minutes = m ? parseInt(m, 10) : 0;
  const seconds = s ? parseInt(s, 10) : 0;
  const total = hours * 3600 + minutes * 60 + seconds;
  if (total <= 0) return null;
  return { mode: 'countdown', durationSeconds: total };
}

/** Format a number of seconds as MM:SS, or HH:MM:SS if >= 1h. */
export function formatDuration(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = s % 60;
  const pad = (n: number) => n.toString().padStart(2, '0');
  if (hours > 0) return `${hours}:${pad(minutes)}:${pad(seconds)}`;
  return `${pad(minutes)}:${pad(seconds)}`;
}
