<script lang="ts">
  import { untrack } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { formatDuration, parseTimerInput } from '../utils/time-parser';
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

  // "When timer ends" config — read from settings on open, persisted as
  // last-used so the next timer start pre-fills with the same values.
  let actionEnabled = $state(false);
  let actionPath = $state('');
  let actionArgs = $state('');
  let pomoCycles = $state(4);

  // Seed the form from saved settings only when the popover transitions
  // to open. Reads inside `untrack` don't become tracked dependencies, so
  // typing in the URL field (or picking an app) won't re-fire this effect
  // and overwrite the user's edits. The only tracked dep is `open`.
  $effect(() => {
    if (!open) return;
    untrack(() => {
      actionPath = settingsStore.lastActionPath;
      actionArgs = settingsStore.lastActionArgs;
      pomoCycles = settingsStore.lastPomodoroCycles || 4;
      actionEnabled = actionPath !== '' || actionArgs !== '';
    });
  });

  function appLabel(path: string): string {
    if (!path) return '';
    const parts = path.split(/[\\/]/);
    return parts[parts.length - 1] || path;
  }

  async function pickApp() {
    try {
      const sel = await openDialog({
        directory: false,
        multiple: false,
        title: 'Choose an app to open when the timer ends',
        filters: [{ name: 'Application', extensions: ['app', 'exe'] }],
      });
      if (typeof sel === 'string') {
        actionPath = sel;
      }
    } catch (e) {
      error = String(e);
    }
  }

  function clearApp() {
    actionPath = '';
  }

  function buildActionOptions(mode: 'countdown' | 'pomodoro' | 'stopwatch') {
    return {
      actionPath: actionEnabled ? actionPath.trim() || null : null,
      actionArgs: actionEnabled ? actionArgs.trim() || null : null,
      // Cycle limit only applies when an action is configured for pomodoro;
      // a plain pomodoro without an action keeps cycling indefinitely as
      // before.
      pomodoroCycles: actionEnabled && mode === 'pomodoro' ? pomoCycles : null,
    };
  }

  async function persistLastUsed(mode: 'countdown' | 'pomodoro' | 'stopwatch') {
    if (actionEnabled) {
      await settingsStore.setLastActionPath(actionPath.trim());
      await settingsStore.setLastActionArgs(actionArgs.trim());
    } else {
      // Don't wipe the saved app path on every "off" — keep it so toggling
      // back on restores the previous selection.
    }
    if (mode === 'pomodoro') {
      await settingsStore.setLastPomodoroCycles(pomoCycles);
    }
  }

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

  // Live preview of the custom input — guides the user to a valid syntax
  // before they click Start.
  let customParsed = $derived.by(() => {
    const v = custom.trim();
    if (!v) return null;
    return parseTimerInput(v);
  });
  let customValid = $derived(customParsed !== null);
  let customPreview = $derived.by(() => {
    if (!customParsed) return null;
    if (customParsed.mode === 'countdown' && customParsed.durationSeconds) {
      const d = formatDuration(customParsed.durationSeconds);
      return customParsed.targetTime
        ? `Countdown · until ${customParsed.targetTime} (in ${d})`
        : `Countdown · ${d}`;
    }
    if (customParsed.mode === 'pomodoro') return 'Pomodoro · 25 work / 5 break';
    if (customParsed.mode === 'stopwatch') return 'Stopwatch · counts up';
    return null;
  });

  async function startPreset(p: Preset) {
    error = null;
    try {
      await tauri.startTimer(noteId, p.mode, p.seconds, buildActionOptions(p.mode));
      await settingsStore.setLastTimerPreset(p.id);
      await persistLastUsed(p.mode);
      await onStarted();
      onClose();
    } catch (e) {
      error = String(e);
    }
  }

  async function startCustom() {
    error = null;
    if (!customParsed) {
      error = 'Try "25m", "1h30m", "90s", "pomo", or "stopwatch"';
      return;
    }
    try {
      await tauri.startTimer(
        noteId,
        customParsed.mode,
        customParsed.durationSeconds,
        buildActionOptions(customParsed.mode),
      );
      const matchingPreset = presets.find((p) => p.id === custom.trim().toLowerCase());
      await settingsStore.setLastTimerPreset(matchingPreset?.id ?? '');
      await persistLastUsed(customParsed.mode);
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

  function onWindowKeyDown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape') {
      onClose();
      return;
    }
    // Number keys 1-6 trigger the corresponding preset, but only when the
    // user isn't typing in the custom input.
    const t = e.target as HTMLElement | null;
    const typingInInput = t?.tagName === 'INPUT' || t?.tagName === 'TEXTAREA';
    if (typingInInput) return;
    const num = Number(e.key);
    if (Number.isInteger(num) && num >= 1 && num <= presets.length) {
      e.preventDefault();
      void startPreset(presets[num - 1]);
    }
  }

  function onSubmit(e: Event) {
    e.preventDefault();
    void startCustom();
  }
</script>

<svelte:window onmousedown={onWindowMouseDown} onkeydown={onWindowKeyDown} />

{#if open}
  <div class="popover" bind:this={popoverEl} role="dialog" aria-label="Set timer">
    <div class="section">
      <div class="section-heading">Quick start</div>
      <div class="presets">
        {#each presets as p, i (p.id)}
          <button
            class="preset"
            class:last-used={lastPreset === p.id}
            onclick={() => startPreset(p)}
            aria-label={`${p.label} ${p.sub ?? ''}${lastPreset === p.id ? ' — last used' : ''}`}
            title={lastPreset === p.id ? 'Last used' : undefined}
          >
            <span class="preset-key" aria-hidden="true">{i + 1}</span>
            {#if lastPreset === p.id}
              <span class="last-dot" aria-hidden="true"></span>
            {/if}
            <span class="preset-num">{p.label}</span>
            {#if p.sub}<span class="preset-sub">{p.sub}</span>{/if}
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
          placeholder="25m, 1h30m, 14:30, 2:30pm…"
          bind:value={custom}
          autofocus
          aria-label="Custom timer"
          class:invalid={custom.trim() !== '' && !customValid}
        />
        <button type="submit" class="start-btn" disabled={!customValid}>Start</button>
      </form>
      {#if customPreview}
        <p class="preview valid" aria-live="polite">→ {customPreview}</p>
      {:else if custom.trim() !== '' && !customValid}
        <p class="preview invalid" aria-live="polite">Not a valid timer</p>
      {:else}
        <p class="hint">Duration (25m · 1h30m · 90s) or clock time (14:30 · 2:30pm)</p>
      {/if}
      {#if error}<p class="error">{error}</p>{/if}
    </div>

    <div class="divider" aria-hidden="true"></div>

    <div class="section">
      <label class="action-toggle">
        <input type="checkbox" bind:checked={actionEnabled} />
        <span>When it ends, open…</span>
      </label>

      {#if actionEnabled}
        <div class="action-row">
          <button type="button" class="pick-btn" onclick={pickApp}>📎 Pick app…</button>
          {#if actionPath}
            <span class="app-chip" title={actionPath}>
              {appLabel(actionPath)}
              <button type="button" class="chip-x" onclick={clearApp} aria-label="Clear app"
                >×</button
              >
            </span>
          {/if}
        </div>
        <input
          type="text"
          class="url-input"
          placeholder="URL or args (optional)"
          bind:value={actionArgs}
          aria-label="URL or arguments to pass"
        />
        <p class="hint">
          {#if actionPath && actionArgs}
            Opens the app with the URL/args.
          {:else if actionPath}
            Just opens the app.
          {:else if actionArgs}
            Opens URL in default browser.
          {:else}
            Pick an app, type a URL, or both.
          {/if}
        </p>
        <div class="action-row cycles-row">
          <span class="cycles-label">Pomodoro cycles</span>
          <button
            type="button"
            class="step-btn"
            onclick={() => (pomoCycles = Math.max(1, pomoCycles - 1))}
            aria-label="Decrease cycles">−</button
          >
          <span class="cycles-value" aria-live="polite">{pomoCycles}</span>
          <button
            type="button"
            class="step-btn"
            onclick={() => (pomoCycles = Math.min(8, pomoCycles + 1))}
            aria-label="Increase cycles">+</button
          >
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .popover {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 6px;
    width: 232px;
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
    height: 48px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.04);
    border: 1px solid transparent;
    cursor: pointer;
    overflow: hidden;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out,
      transform 80ms ease-out;
  }
  .preset:hover {
    background: rgba(216, 90, 48, 0.08);
    border-color: rgba(216, 90, 48, 0.35);
  }
  .preset:active {
    transform: scale(0.97);
  }
  .preset.last-used {
    border-color: var(--accent);
    background: rgba(216, 90, 48, 0.07);
  }
  .preset-key {
    position: absolute;
    top: 3px;
    left: 5px;
    font-size: 9px;
    font-weight: 600;
    color: rgba(0, 0, 0, 0.32);
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }
  .last-dot {
    position: absolute;
    top: 5px;
    right: 5px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
  }
  .preset-num {
    font-size: 15px;
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
    align-items: stretch;
    width: 100%;
  }
  .custom-form input {
    flex: 1 1 auto;
    /* min-width: 0 lets the input shrink below its placeholder's natural
       width — without this the long placeholder pushes the Start button
       past the popover's right edge. */
    min-width: 0;
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
  .custom-form input.invalid {
    border-color: rgba(216, 90, 48, 0.45);
    background: rgba(216, 90, 48, 0.05);
  }
  .start-btn {
    flex-shrink: 0;
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

  .preview,
  .hint,
  .error {
    margin: 0;
    font-size: 10px;
    line-height: 1.4;
    font-variant-numeric: tabular-nums;
  }
  .preview.valid {
    color: var(--accent);
    font-weight: 500;
  }
  .preview.invalid {
    color: var(--accent);
    opacity: 0.85;
  }
  .hint {
    color: rgba(0, 0, 0, 0.4);
  }
  .error {
    color: var(--accent);
  }

  .action-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    user-select: none;
  }
  .action-toggle input[type='checkbox'] {
    accent-color: var(--accent);
    cursor: pointer;
  }
  .action-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
  }
  .pick-btn {
    height: 24px;
    padding: 0 8px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.05);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 120ms ease-out;
  }
  .pick-btn:hover {
    background: rgba(216, 90, 48, 0.12);
  }
  .app-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 4px 0 8px;
    border-radius: 11px;
    background: rgba(216, 90, 48, 0.12);
    color: var(--accent);
    font-size: 11px;
    font-weight: 500;
    max-width: 140px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .chip-x {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: transparent;
    color: var(--accent);
    font-size: 13px;
    line-height: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  .chip-x:hover {
    background: rgba(216, 90, 48, 0.2);
  }
  .url-input {
    width: 100%;
    height: 26px;
    border-radius: 5px;
    background: rgba(0, 0, 0, 0.04);
    border: 1px solid transparent;
    padding: 0 8px;
    font-size: 11px;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out;
  }
  .url-input:focus {
    outline: none;
    background: white;
    border-color: rgba(216, 90, 48, 0.45);
  }
  .cycles-row {
    margin-top: 2px;
  }
  .cycles-label {
    flex: 1;
    font-size: 11px;
    color: rgba(0, 0, 0, 0.6);
  }
  .step-btn {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.05);
    font-size: 13px;
    line-height: 1;
    cursor: pointer;
    transition: background-color 120ms ease-out;
  }
  .step-btn:hover {
    background: rgba(216, 90, 48, 0.15);
    color: var(--accent);
  }
  .cycles-value {
    min-width: 18px;
    text-align: center;
    font-size: 12px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
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
    .preset-key {
      color: rgba(255, 255, 255, 0.4);
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
    .hint {
      color: rgba(255, 255, 255, 0.4);
    }
    .pick-btn,
    .step-btn {
      background: rgba(255, 255, 255, 0.06);
      color: inherit;
    }
    .url-input {
      background: rgba(255, 255, 255, 0.06);
      color: inherit;
    }
    .url-input:focus {
      background: rgba(255, 255, 255, 0.1);
    }
    .cycles-label {
      color: rgba(255, 255, 255, 0.6);
    }
  }
</style>
