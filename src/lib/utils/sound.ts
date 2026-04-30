// Web Audio API synthesizer for the timer-done chime.
// We synthesize on demand instead of bundling an audio file:
// - no asset to ship
// - works offline
// - doesn't require any extra Tauri capability

export type SoundChoice = 'bell' | 'soft' | 'bright';

export const SOUND_CHOICES: ReadonlyArray<{ id: SoundChoice; label: string; description: string }> =
  [
    { id: 'bell', label: 'Bell', description: 'Classic two-note ding (default)' },
    { id: 'soft', label: 'Soft', description: 'Lower, gentler — won’t startle' },
    { id: 'bright', label: 'Bright', description: 'Higher and crisp — hard to miss' },
  ] as const;

let ctx: AudioContext | null = null;

function getContext(): AudioContext | null {
  if (typeof window === 'undefined') return null;
  if (!ctx) {
    const Ctor =
      window.AudioContext ??
      (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!Ctor) return null;
    ctx = new Ctor();
  }
  if (ctx.state === 'suspended') {
    void ctx.resume();
  }
  return ctx;
}

interface ToneOptions {
  type?: OscillatorType;
  peakGain?: number;
}

function ringTone(
  frequency: number,
  startAt: number,
  durationSec: number,
  { type = 'sine', peakGain = 0.18 }: ToneOptions = {},
): void {
  const audio = getContext();
  if (!audio) return;

  const osc = audio.createOscillator();
  osc.type = type;
  osc.frequency.value = frequency;

  const gain = audio.createGain();
  gain.gain.setValueAtTime(0, startAt);
  gain.gain.linearRampToValueAtTime(peakGain, startAt + 0.01);
  gain.gain.exponentialRampToValueAtTime(0.0001, startAt + durationSec);

  osc.connect(gain).connect(audio.destination);
  osc.start(startAt);
  osc.stop(startAt + durationSec + 0.05);
}

function playBell(): void {
  const audio = getContext();
  if (!audio) return;
  const now = audio.currentTime;
  ringTone(880, now, 0.6); // A5
  ringTone(587.33, now + 0.18, 0.9, { peakGain: 0.14 }); // D5
}

function playSoft(): void {
  const audio = getContext();
  if (!audio) return;
  const now = audio.currentTime;
  // Lower octave, triangle wave for a warmer timbre, longer tails.
  ringTone(523.25, now, 1.1, { type: 'triangle', peakGain: 0.12 }); // C5
  ringTone(392.0, now + 0.22, 1.4, { type: 'triangle', peakGain: 0.1 }); // G4
}

function playBright(): void {
  const audio = getContext();
  if (!audio) return;
  const now = audio.currentTime;
  // Higher fundamentals, three quick pings — crisp and attention-getting.
  ringTone(1318.51, now, 0.35, { peakGain: 0.16 }); // E6
  ringTone(1567.98, now + 0.12, 0.4, { peakGain: 0.14 }); // G6
  ringTone(2093.0, now + 0.24, 0.55, { peakGain: 0.12 }); // C7
}

/** Play the timer-done chime once, for the given variant. */
export function playTimerDone(choice: SoundChoice = 'bell'): void {
  switch (choice) {
    case 'soft':
      playSoft();
      return;
    case 'bright':
      playBright();
      return;
    case 'bell':
    default:
      playBell();
      return;
  }
}

let loopHandle: ReturnType<typeof setInterval> | null = null;

/** Start chiming every 1.8s until `stopTimerDoneLoop()` is called. */
export function startTimerDoneLoop(choice: SoundChoice = 'bell'): void {
  stopTimerDoneLoop();
  playTimerDone(choice);
  loopHandle = setInterval(() => playTimerDone(choice), 1800);
}

export function stopTimerDoneLoop(): void {
  if (loopHandle !== null) {
    clearInterval(loopHandle);
    loopHandle = null;
  }
}
