<script lang="ts">
  import { formatDuration } from '../utils/time-parser';
  import { tauri } from '../utils/tauri';
  import type { TimerStatePayload } from '../types';
  import TimerPopover from './TimerPopover.svelte';

  interface Props {
    noteId: string;
    timer: TimerStatePayload | null;
    flashing: boolean;
    onChange: () => void | Promise<void>;
  }

  let { noteId, timer, flashing, onChange }: Props = $props();

  let popoverOpen = $state(false);

  function display(): string {
    if (!timer) return '';
    if (timer.mode === 'stopwatch') return formatDuration(timer.elapsed_seconds);
    if (timer.remaining_seconds !== null) return formatDuration(timer.remaining_seconds);
    return formatDuration(timer.elapsed_seconds);
  }

  function statusLabel(): string {
    if (!timer) return '';
    const phase = timer.pomodoro_phase ? ` · ${timer.pomodoro_phase}` : '';
    return `${timer.state}${phase}`;
  }

  async function pause() {
    if (timer?.state === 'paused') await tauri.resumeTimer(noteId);
    else await tauri.pauseTimer(noteId);
    await onChange();
  }

  async function cancel() {
    await tauri.cancelTimer(noteId);
    await onChange();
  }

  // Snooze: cancel the just-fired alarm and start a fresh countdown for
  // the chosen offset. The new timer is a plain countdown — no post-action
  // and no calendar-event accounting, so this works the same for editable
  // notes and event-backed notes alike.
  async function snooze(seconds: number) {
    await tauri.cancelTimer(noteId);
    await tauri.startTimer(noteId, 'countdown', seconds);
    await onChange();
  }
</script>

<div class="timer-bar" class:flashing>
  {#if timer && timer.state === 'done'}
    <span class="display done">⏱ Done</span>
    <span class="spacer"></span>
    <button class="snooze" onclick={() => snooze(5 * 60)} title="Snooze 5 minutes">+5m</button>
    <button class="snooze" onclick={() => snooze(15 * 60)} title="Snooze 15 minutes">+15m</button>
    <button class="snooze" onclick={() => snooze(60 * 60)} title="Snooze 1 hour">+1h</button>
    <button class="dismiss" onclick={cancel} aria-label="Dismiss timer">Dismiss</button>
  {:else if timer && (timer.state === 'running' || timer.state === 'paused')}
    <span class="display">⏱ {display()}</span>
    <span class="status">· {statusLabel()}</span>
    <span class="spacer"></span>
    <button
      class="link-btn"
      onclick={pause}
      aria-label={timer.state === 'paused' ? 'Resume' : 'Pause'}
    >
      {timer.state === 'paused' ? 'Resume' : 'Pause'}
    </button>
    <button class="link-btn" onclick={cancel} aria-label="Cancel timer">Cancel</button>
  {:else}
    <button
      class="set-btn"
      class:active={popoverOpen}
      onclick={() => (popoverOpen = !popoverOpen)}
      aria-label="Set timer"
      aria-expanded={popoverOpen}
    >
      <svg viewBox="0 0 16 16" width="11" height="11" aria-hidden="true">
        <circle cx="8" cy="8.5" r="5.5" fill="none" stroke="currentColor" stroke-width="1.2" />
        <path
          d="M8 5.5V8.5L9.8 9.6"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
        />
        <path
          d="M6.5 2L8 1L9.5 2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
        />
      </svg>
      <span>Set timer</span>
    </button>
    <TimerPopover
      {noteId}
      open={popoverOpen}
      onClose={() => (popoverOpen = false)}
      onStarted={onChange}
    />
  {/if}
</div>

<style>
  .timer-bar {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 24px;
    padding: 0 10px;
    font-size: 11px;
    flex-shrink: 0;
  }
  .timer-bar.flashing {
    animation: flash 0.5s ease-in-out 6;
  }
  @keyframes flash {
    0%,
    100% {
      background: transparent;
    }
    50% {
      background: rgba(216, 90, 48, 0.35);
    }
  }
  .display {
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }
  .display.done {
    color: var(--accent);
    font-weight: 600;
  }
  .status {
    color: rgba(0, 0, 0, 0.55);
  }
  .spacer {
    flex: 1;
  }
  .link-btn {
    color: var(--accent);
    font-size: 11px;
    font-weight: 500;
    padding: 0 4px;
  }
  .link-btn:hover {
    text-decoration: underline;
  }

  .set-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 20px;
    padding: 0 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    color: var(--accent);
    cursor: pointer;
    transition: background-color 120ms ease-out;
  }
  .set-btn:hover,
  .set-btn.active {
    background: rgba(216, 90, 48, 0.1);
  }

  .dismiss {
    color: white;
    background: var(--accent);
    padding: 2px 10px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 120ms ease-out;
  }
  .dismiss:hover {
    background: #c64f29;
  }

  .snooze {
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 500;
    color: var(--accent);
    background: rgba(216, 90, 48, 0.08);
    cursor: pointer;
    transition: background-color 120ms ease-out;
    font-variant-numeric: tabular-nums;
  }
  .snooze:hover {
    background: rgba(216, 90, 48, 0.18);
  }

  @media (prefers-color-scheme: dark) {
    .status {
      color: rgba(255, 255, 255, 0.55);
    }
    .snooze {
      background: rgba(216, 90, 48, 0.18);
    }
    .snooze:hover {
      background: rgba(216, 90, 48, 0.28);
    }
  }
</style>
