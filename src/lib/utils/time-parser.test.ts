import { describe, expect, it } from 'vitest';
import { formatDuration, parseTimerInput } from './time-parser';

describe('parseTimerInput', () => {
  it('parses minute-only countdowns', () => {
    expect(parseTimerInput('25m')).toEqual({ mode: 'countdown', durationSeconds: 1500 });
  });

  it('parses hour+minute countdowns', () => {
    expect(parseTimerInput('1h30m')).toEqual({ mode: 'countdown', durationSeconds: 5400 });
  });

  it('parses hour-only countdowns', () => {
    expect(parseTimerInput('2h')).toEqual({ mode: 'countdown', durationSeconds: 7200 });
  });

  it('parses second-only countdowns', () => {
    expect(parseTimerInput('90s')).toEqual({ mode: 'countdown', durationSeconds: 90 });
  });

  it('parses combined h+m+s', () => {
    expect(parseTimerInput('1h1m30s')).toEqual({ mode: 'countdown', durationSeconds: 3690 });
  });

  it('treats "pomo" and "pomodoro" as pomodoro mode', () => {
    expect(parseTimerInput('pomo')).toEqual({ mode: 'pomodoro', durationSeconds: 1500 });
    expect(parseTimerInput('pomodoro')).toEqual({ mode: 'pomodoro', durationSeconds: 1500 });
  });

  it('treats "stopwatch", "sw", and empty input as stopwatch', () => {
    expect(parseTimerInput('stopwatch')).toEqual({ mode: 'stopwatch', durationSeconds: null });
    expect(parseTimerInput('sw')).toEqual({ mode: 'stopwatch', durationSeconds: null });
    expect(parseTimerInput('')).toEqual({ mode: 'stopwatch', durationSeconds: null });
  });

  it('is case-insensitive and ignores surrounding whitespace', () => {
    expect(parseTimerInput('  25M ')).toEqual({ mode: 'countdown', durationSeconds: 1500 });
    expect(parseTimerInput('POMO')).toEqual({ mode: 'pomodoro', durationSeconds: 1500 });
  });

  it('rejects garbage input', () => {
    expect(parseTimerInput('abc')).toBeNull();
    expect(parseTimerInput('25x')).toBeNull();
    expect(parseTimerInput('1h2x3m')).toBeNull();
  });

  it('rejects 0-duration countdowns', () => {
    expect(parseTimerInput('0m')).toBeNull();
    expect(parseTimerInput('0h0m0s')).toBeNull();
  });

  describe('clock-time inputs', () => {
    // Anchor "now" at 2026-04-30 12:00:00 local time so the math is deterministic.
    const NOW = new Date(2026, 3, 30, 12, 0, 0);

    it('parses 24h "HH:MM" into a countdown to that time today', () => {
      const r = parseTimerInput('14:30', NOW);
      // 14:30 - 12:00 = 2h30m = 9000s
      expect(r).toEqual({ mode: 'countdown', durationSeconds: 9000, targetTime: '14:30' });
    });

    it('rolls a past time to the next day', () => {
      const r = parseTimerInput('09:00', NOW);
      // 09:00 today already passed at 12:00, fire next day → 21h = 75600s
      expect(r).toEqual({ mode: 'countdown', durationSeconds: 21 * 3600, targetTime: '09:00' });
    });

    it('accepts the @ prefix', () => {
      expect(parseTimerInput('@14:30', NOW)).toEqual({
        mode: 'countdown',
        durationSeconds: 9000,
        targetTime: '14:30',
      });
    });

    it('accepts the "at" prefix', () => {
      expect(parseTimerInput('at 14:30', NOW)).toEqual({
        mode: 'countdown',
        durationSeconds: 9000,
        targetTime: '14:30',
      });
    });

    it('parses 12h with am/pm', () => {
      expect(parseTimerInput('2:30pm', NOW)).toEqual({
        mode: 'countdown',
        durationSeconds: 9000,
        targetTime: '14:30',
      });
      expect(parseTimerInput('9am', NOW)).toEqual({
        mode: 'countdown',
        durationSeconds: 21 * 3600,
        targetTime: '09:00',
      });
      expect(parseTimerInput('12am', NOW)).toEqual({
        mode: 'countdown',
        durationSeconds: 12 * 3600,
        targetTime: '00:00',
      });
      expect(parseTimerInput('12pm', NOW)).toEqual({
        mode: 'countdown',
        durationSeconds: 24 * 3600,
        targetTime: '12:00',
      });
    });

    it('rejects out-of-range clock times', () => {
      expect(parseTimerInput('25:00', NOW)).toBeNull();
      expect(parseTimerInput('14:60', NOW)).toBeNull();
      expect(parseTimerInput('13pm', NOW)).toBeNull();
    });
  });
});

describe('formatDuration', () => {
  it('formats sub-hour durations as MM:SS', () => {
    expect(formatDuration(0)).toBe('00:00');
    expect(formatDuration(59)).toBe('00:59');
    expect(formatDuration(60)).toBe('01:00');
    expect(formatDuration(125)).toBe('02:05');
    expect(formatDuration(3599)).toBe('59:59');
  });

  it('formats >= 1h durations as H:MM:SS', () => {
    expect(formatDuration(3600)).toBe('1:00:00');
    expect(formatDuration(3725)).toBe('1:02:05');
  });

  it('clamps negative inputs to zero', () => {
    expect(formatDuration(-5)).toBe('00:00');
  });
});
