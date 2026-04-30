// Web Audio API synthesizer for the timer-done chime.
// We synthesize on demand instead of bundling an audio file:
// - no asset to ship
// - works offline
// - doesn't require any extra Tauri capability
//
// The chime is a two-note bell: a higher note fades out while a lower one
// rings underneath, giving a soft "ding-dong" without being startling.

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
  // Some browsers/WebViews start the context in "suspended" state until a
  // user gesture; resume() is a no-op if already running.
  if (ctx.state === 'suspended') {
    void ctx.resume();
  }
  return ctx;
}

function ringTone(frequency: number, startAt: number, durationSec: number, peakGain = 0.18): void {
  const audio = getContext();
  if (!audio) return;

  const osc = audio.createOscillator();
  osc.type = 'sine';
  osc.frequency.value = frequency;

  const gain = audio.createGain();
  // Quick attack, exponential decay — a bell-like envelope.
  gain.gain.setValueAtTime(0, startAt);
  gain.gain.linearRampToValueAtTime(peakGain, startAt + 0.01);
  gain.gain.exponentialRampToValueAtTime(0.0001, startAt + durationSec);

  osc.connect(gain).connect(audio.destination);
  osc.start(startAt);
  osc.stop(startAt + durationSec + 0.05);
}

/** Play a soft two-note chime — used when a countdown or pomodoro phase completes. */
export function playTimerDone(): void {
  const audio = getContext();
  if (!audio) return;
  const now = audio.currentTime;
  ringTone(880, now, 0.6); // A5
  ringTone(587.33, now + 0.18, 0.9, 0.14); // D5, slightly later, longer tail
}
