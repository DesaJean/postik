<script lang="ts">
  import { formatDuration, parseTimerInput } from '../utils/time-parser';
  import { tauri } from '../utils/tauri';
  import type { TimerStatePayload } from '../types';

  interface Props {
    noteId: string;
    timer: TimerStatePayload | null;
    flashing: boolean;
  }

  let { noteId, timer, flashing }: Props = $props();

  let editing = $state(false);
  let input = $state('');
  let error = $state<string | null>(null);

  function display(): string {
    if (!timer) return '';
    if (timer.mode === 'stopwatch') {
      return formatDuration(timer.elapsed_seconds);
    }
    if (timer.remaining_seconds !== null) {
      return formatDuration(timer.remaining_seconds);
    }
    return formatDuration(timer.elapsed_seconds);
  }

  function statusLabel(): string {
    if (!timer) return '';
    const phase = timer.pomodoro_phase ? ` · ${timer.pomodoro_phase}` : '';
    return `${timer.state}${phase}`;
  }

  async function submit() {
    error = null;
    const parsed = parseTimerInput(input);
    if (!parsed) {
      error = 'Try "25m", "1h30m", "90s", "pomo", or "stopwatch"';
      return;
    }
    try {
      await tauri.startTimer(noteId, parsed.mode, parsed.durationSeconds);
      editing = false;
      input = '';
    } catch (e) {
      error = String(e);
    }
  }

  async function pause() {
    if (timer?.state === 'paused') await tauri.resumeTimer(noteId);
    else await tauri.pauseTimer(noteId);
  }

  async function cancel() {
    await tauri.cancelTimer(noteId);
  }
</script>

<div class="timer-bar" class:flashing>
  {#if editing}
    <form
      class="timer-input-form"
      onsubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        placeholder="25m, 1h30m, 90s, pomo, or stopwatch"
        bind:value={input}
        autofocus
        onblur={() => {
          if (!input) editing = false;
        }}
        onkeydown={(e) => e.key === 'Escape' && (editing = false)}
      />
      <button type="submit" class="link-btn">Set</button>
    </form>
    {#if error}<span class="error">{error}</span>{/if}
  {:else if timer && timer.state !== 'idle' && timer.state !== 'done'}
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
    <button class="link-btn" onclick={() => (editing = true)} aria-label="Set timer"
      >Set timer</button
    >
  {/if}
</div>

<style>
  .timer-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 24px;
    padding: 0 10px;
    font-size: 11px;
    border-top: 1px solid rgba(0, 0, 0, 0.06);
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
  }
  .link-btn:hover {
    text-decoration: underline;
  }
  .timer-input-form {
    display: flex;
    flex: 1;
    gap: 6px;
    align-items: center;
  }
  .timer-input-form input {
    flex: 1;
    border: none;
    background: rgba(0, 0, 0, 0.05);
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 11px;
  }
  .timer-input-form input:focus {
    outline: 1px solid var(--accent);
  }
  .error {
    color: var(--accent);
    font-size: 10px;
  }
</style>
