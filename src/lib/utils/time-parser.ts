import type { TimerMode } from '../types';

export interface ParsedTimer {
  mode: TimerMode;
  durationSeconds: number | null;
  /** Set when the user expressed the timer as a clock time (e.g. "14:30").
   * The popover uses it to render "until 14:30" instead of just the
   * computed remaining duration. The backend never sees this — the field
   * is informational only. */
  targetTime?: string;
}

const COUNTDOWN_RE = /^(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?$/;
const CLOCK_24H_RE = /^(?:@|at\s+)?(\d{1,2}):(\d{2})$/;
const CLOCK_12H_RE = /^(?:@|at\s+)?(\d{1,2})(?::(\d{2}))?\s*(am|pm)$/;

/**
 * Parse a user-typed timer string into a normalized command.
 *
 * Accepts:
 *   "25m"        → countdown 1500s
 *   "1h30m"      → countdown 5400s
 *   "90s"        → countdown 90s
 *   "2h"         → countdown 7200s
 *   "14:30"      → countdown to next 14:30 (today, or tomorrow if past)
 *   "@14:30"     → same
 *   "at 14:30"   → same
 *   "2:30pm"     → countdown to next 14:30
 *   "9am"        → countdown to next 09:00
 *   "pomo"       → pomodoro
 *   "stopwatch"  → stopwatch
 *
 * Returns null for unparseable input. An empty string is treated as stopwatch.
 *
 * `now` is injectable for deterministic tests; defaults to the current time.
 */
export function parseTimerInput(raw: string, now: Date = new Date()): ParsedTimer | null {
  const input = raw.trim().toLowerCase();
  if (input === '' || input === 'stopwatch' || input === 'sw') {
    return { mode: 'stopwatch', durationSeconds: null };
  }
  if (input === 'pomo' || input === 'pomodoro') {
    return { mode: 'pomodoro', durationSeconds: 25 * 60 };
  }

  const clock = parseClockTime(input, now);
  if (clock) return clock;

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

/** Resolve a clock-time input into a countdown to the next occurrence of
 * that time. Returns null when the input isn't a recognized clock format. */
function parseClockTime(input: string, now: Date): ParsedTimer | null {
  let hour: number;
  let minute: number;

  const m24 = CLOCK_24H_RE.exec(input);
  if (m24) {
    hour = parseInt(m24[1], 10);
    minute = parseInt(m24[2], 10);
  } else {
    const m12 = CLOCK_12H_RE.exec(input);
    if (!m12) return null;
    hour = parseInt(m12[1], 10);
    minute = m12[2] ? parseInt(m12[2], 10) : 0;
    if (hour < 1 || hour > 12) return null;
    // 12am = 00:00, 12pm = 12:00, 1pm-11pm add 12.
    if (hour === 12) hour = 0;
    if (m12[3] === 'pm') hour += 12;
  }

  if (hour < 0 || hour >= 24 || minute < 0 || minute >= 60) return null;

  const target = new Date(now);
  target.setHours(hour, minute, 0, 0);
  let diff = Math.floor((target.getTime() - now.getTime()) / 1000);
  if (diff <= 0) diff += 24 * 60 * 60; // already passed today → fire next day
  return {
    mode: 'countdown',
    durationSeconds: diff,
    targetTime: `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`,
  };
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
