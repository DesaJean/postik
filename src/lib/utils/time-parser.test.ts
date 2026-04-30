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
