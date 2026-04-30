<script lang="ts">
  import { parseTimerInput } from '../utils/time-parser';
  import { tauri } from '../utils/tauri';
  import { settingsStore } from '../stores/settings.svelte';

  interface Props {
    noteId: string;
    open: boolean;
    onClose: () => void;
    onStarted: () => void | Promise<void>;
  }

  let { noteId, open, onClose, onStarted }: Props = $props();

  let custom = $state('');
  let error = $state<string | null>(null);
  let popoverEl: HTMLDivElement | undefined = $state();

  interface Preset {
    id: string;
    label: string;
    sub?: string;
    mode: 'countdown' | 'pomodoro' | 'stopwatch';
    seconds: number | null;
  }

  const presets: Preset[] = [
    { id: '5m', label: '5', sub: 'min', mode: 'countdown', seconds: 5 * 60 },
    { id: '15m', label: '15', sub: 'min', mode: 'countdown', seconds: 15 * 60 },
    { id: '25m', label: '25', sub: 'min', mode: 'countdown', seconds: 25 * 60 },
    { id: '50m', label: '50', sub: 'min', mode: 'countdown', seconds: 50 * 60 },
    { id: 'pomo', label: 'Pomo', sub: '25/5', mode: 'pomodoro', seconds: 25 * 60 },
    { id: 'sw', label: '∞', sub: 'stopwatch', mode: 'stopwatch', seconds: null },
  ];

  let lastPreset = $derived(settingsStore.lastTimerPreset);

  async function startPreset(p: Preset) {
    error = null;
    try {
      await tauri.startTimer(noteId, p.mode, p.seconds);
      await settingsStore.setLastTimerPreset(p.id);
      await onStarted();
      onClose();
    } catch (e) {
      error = String(e);
    }
  }

  async function startCustom() {
    error = null;
    const parsed = parseTimerInput(custom);
    if (!parsed) {
      error = 'Try "25m", "1h30m", "90s", "pomo", or "stopwatch"';
      return;
    }
    try {
      await tauri.startTimer(noteId, parsed.mode, parsed.durationSeconds);
      // Custom inputs that match a preset's id (e.g. "25m") get attributed to
      // that preset; otherwise we clear lastPreset since the input isn't a
      // canonical preset we can highlight.
      const matchingPreset = presets.find((p) => p.id === custom.trim().toLowerCase());
      await settingsStore.setLastTimerPreset(matchingPreset?.id ?? '');
      custom = '';
      await onStarted();
      onClose();
    } catch (e) {
      error = String(e);
    }
  }

  function onWindowMouseDown(e: MouseEvent) {
    if (!open) return;
    const t = e.target as Node;
    if (popoverEl && !popoverEl.contains(t)) onClose();
  }

  function onSubmit(e: Event) {
    e.preventDefault();
    void startCustom();
  }
</script>

<svelte:window onmousedown={onWindowMouseDown} onkeydown={(e) => e.key === 'Escape' && onClose()} />

{#if open}
  <div class="popover" bind:this={popoverEl} role="dialog" aria-label="Set timer">
    <div class="section">
      <div class="section-heading">Quick start</div>
      <div class="presets">
        {#each presets as p (p.id)}
          <button
            class="preset"
            class:last-used={lastPreset === p.id}
            onclick={() => startPreset(p)}
            aria-label={`${p.label} ${p.sub ?? ''}${lastPreset === p.id ? ' — last used' : ''}`}
            title={lastPreset === p.id ? 'Last used' : undefined}
          >
            <span class="preset-num">{p.label}</span>
            {#if p.sub}<span class="preset-sub">{p.sub}</span>{/if}
            {#if lastPreset === p.id}
              <span class="preset-tag" aria-hidden="true">Last</span>
            {/if}
          </button>
        {/each}
      </div>
    </div>

    <div class="divider" aria-hidden="true"></div>

    <div class="section">
      <div class="section-heading">Custom</div>
      <form class="custom-form" onsubmit={onSubmit}>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          type="text"
          placeholder="25m, 1h30m, 90s, pomo…"
          bind:value={custom}
          autofocus
          aria-label="Custom timer"
        />
        <button type="submit" class="start-btn" disabled={!custom.trim()}>Start</button>
      </form>
      {#if error}<p class="error">{error}</p>{/if}
    </div>
  </div>
{/if}

<style>
  .popover {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 6px;
    width: 220px;
    background: rgba(255, 255, 255, 0.98);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 8px;
    box-shadow:
      0 1px 2px rgba(0, 0, 0, 0.04),
      0 8px 24px rgba(0, 0, 0, 0.12);
    padding: 10px 12px 12px;
    z-index: 30;
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    animation: pop-in 140ms cubic-bezier(0.4, 0, 0.2, 1);
  }
  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translateY(2px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-heading {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgba(0, 0, 0, 0.45);
  }
  .divider {
    height: 1px;
    background: rgba(0, 0, 0, 0.08);
    margin: 10px 0;
  }

  .presets {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 5px;
  }
  .preset {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1px;
    height: 44px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.04);
    border: 1px solid transparent;
    cursor: pointer;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out,
      transform 80ms ease-out;
  }
  .preset:hover {
    background: rgba(216, 90, 48, 0.08);
    border-color: rgba(216, 90, 48, 0.3);
  }
  .preset:active {
    transform: scale(0.97);
  }
  .preset.last-used {
    border-color: var(--accent);
    background: rgba(216, 90, 48, 0.06);
  }
  .preset-tag {
    position: absolute;
    top: -5px;
    right: -3px;
    background: var(--accent);
    color: white;
    font-size: 8px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 1px 4px;
    border-radius: 3px;
    line-height: 1.1;
  }
  .preset-num {
    font-size: 14px;
    font-weight: 600;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }
  .preset-sub {
    font-size: 9px;
    font-weight: 500;
    color: rgba(0, 0, 0, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .custom-form {
    display: flex;
    gap: 6px;
  }
  .custom-form input {
    flex: 1;
    height: 28px;
    border-radius: 5px;
    background: rgba(0, 0, 0, 0.04);
    border: 1px solid transparent;
    padding: 0 8px;
    font-size: 12px;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out;
  }
  .custom-form input:focus {
    outline: none;
    background: white;
    border-color: rgba(216, 90, 48, 0.45);
  }
  .start-btn {
    height: 28px;
    padding: 0 12px;
    border-radius: 5px;
    font-size: 11px;
    font-weight: 600;
    background: var(--accent);
    color: white;
    cursor: pointer;
    transition:
      background-color 120ms ease-out,
      opacity 120ms ease-out;
  }
  .start-btn:hover {
    background: #c64f29;
  }
  .start-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    background: rgba(0, 0, 0, 0.18);
  }
  .error {
    font-size: 10px;
    color: var(--accent);
    margin: 0;
  }

  @media (prefers-color-scheme: dark) {
    .popover {
      background: rgba(40, 40, 38, 0.95);
      border-color: rgba(255, 255, 255, 0.1);
      color: #f1f0ec;
    }
    .section-heading,
    .preset-sub {
      color: rgba(255, 255, 255, 0.55);
    }
    .divider {
      background: rgba(255, 255, 255, 0.1);
    }
    .preset {
      background: rgba(255, 255, 255, 0.06);
    }
    .preset:hover {
      background: rgba(216, 90, 48, 0.18);
    }
    .custom-form input {
      background: rgba(255, 255, 255, 0.06);
      color: inherit;
    }
    .custom-form input:focus {
      background: rgba(255, 255, 255, 0.1);
    }
  }
</style>
