<script lang="ts">
  import { onMount } from 'svelte';
  import Switch from './Switch.svelte';
  import { settingsStore } from '../stores/settings.svelte';
  import { SOUND_CHOICES, playTimerDone, type SoundChoice } from '../utils/sound';
  import { checkForUpdate, downloadAndInstall, restart, type UpdateStatus } from '../utils/updater';
  import { tauri } from '../utils/tauri';
  import { eventToAccelerator, prettyAccelerator } from '../utils/keybind';
  import { confirm, open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { notesStore } from '../stores/notes.svelte';

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  // Pomodoro statistics. Loaded on mount and refreshed each time Settings
  // becomes visible — sessions accumulate while the user is in the app.
  let stats = $state<{
    today_seconds: number;
    week_seconds: number;
    last_seven_days: Array<{ date: string; seconds: number }>;
  } | null>(null);
  async function loadStats() {
    try {
      stats = await tauri.pomodoroStats();
    } catch (e) {
      console.error('pomodoro_stats failed', e);
    }
  }
  onMount(loadStats);

  function fmtMinutes(seconds: number): string {
    const m = Math.round(seconds / 60);
    if (m < 60) return `${m}m`;
    const h = Math.floor(m / 60);
    const rem = m % 60;
    return rem === 0 ? `${h}h` : `${h}h ${rem}m`;
  }

  let maxBucketSeconds = $derived(
    Math.max(1, ...(stats?.last_seven_days.map((b) => b.seconds) ?? [0])),
  );

  function dayLabel(iso: string): string {
    // YYYY-MM-DD → 'Mon', 'Tue', etc. (UTC). Last bar is "today".
    const d = new Date(iso + 'T12:00:00Z');
    return d.toLocaleDateString(undefined, { weekday: 'short' });
  }

  function selectSound(id: SoundChoice) {
    settingsStore.setSoundChoice(id);
    // Preview the chosen variant immediately so the user hears what they
    // picked without setting up a real timer.
    playTimerDone(id);
  }

  // Update flow: idle → checking → up-to-date | available → downloading
  // → ready → (user clicks Restart). Errors land in `error` regardless of
  // the prior state.
  let updateStatus = $state<UpdateStatus>({ kind: 'idle' });

  async function onCheckUpdate() {
    updateStatus = { kind: 'checking' };
    try {
      const update = await checkForUpdate();
      if (!update) {
        updateStatus = { kind: 'up-to-date' };
        return;
      }
      updateStatus = {
        kind: 'available',
        version: update.version,
        notes: update.body ?? null,
      };
      // Download and install in the background; restart is user-initiated.
      updateStatus = { kind: 'downloading', downloaded: 0, total: null };
      await downloadAndInstall(update, (downloaded, total) => {
        updateStatus = { kind: 'downloading', downloaded, total };
      });
      updateStatus = { kind: 'ready' };
    } catch (e) {
      updateStatus = { kind: 'error', message: String(e) };
    }
  }

  async function onRestart() {
    await restart();
  }

  // Custom keyboard shortcuts. Loaded on mount; each row enters
  // "recording" mode on click and captures the next valid keypress.
  let shortcuts = $state<
    Array<{ action: string; accelerator: string; default_accelerator: string }>
  >([]);
  let recordingFor = $state<string | null>(null);
  let shortcutError = $state<string | null>(null);

  async function loadShortcuts() {
    try {
      shortcuts = await tauri.listShortcutBindings();
    } catch (e) {
      console.error('list_shortcut_bindings failed', e);
    }
  }

  function shortcutLabel(action: string): string {
    switch (action) {
      case 'new_note':
        return 'New note';
      case 'hide_all':
        return 'Hide / show all';
      case 'start_timer':
        return 'Start timer';
      case 'toggle_pin':
        return 'Toggle pin (focused note)';
      default:
        return action;
    }
  }

  async function onRecordKey(e: KeyboardEvent) {
    if (!recordingFor) return;
    e.preventDefault();
    if (e.key === 'Escape') {
      recordingFor = null;
      shortcutError = null;
      return;
    }
    const accel = eventToAccelerator(e);
    if (!accel) return;
    try {
      await tauri.setShortcut(recordingFor, accel);
      shortcuts = shortcuts.map((s) =>
        s.action === recordingFor ? { ...s, accelerator: accel } : s,
      );
      recordingFor = null;
      shortcutError = null;
    } catch (err) {
      shortcutError = String(err);
    }
  }

  async function resetShortcut(action: string) {
    try {
      const def = await tauri.resetShortcut(action);
      shortcuts = shortcuts.map((s) => (s.action === action ? { ...s, accelerator: def } : s));
    } catch (e) {
      shortcutError = String(e);
    }
  }

  onMount(loadShortcuts);

  // Storage location (D3). The user can point Postik at a custom DB
  // path (e.g. inside iCloud / Dropbox for cross-device sync). Takes
  // effect on next launch.
  let dbPath = $state<string>('');
  let dbStatus = $state<string | null>(null);
  async function loadDbPath() {
    try {
      dbPath = await tauri.currentDbPath();
    } catch (e) {
      console.error('current_db_path failed', e);
    }
  }
  onMount(loadDbPath);

  async function pickDbPath() {
    dbStatus = null;
    try {
      const picked = await saveDialog({
        defaultPath: 'postik.db',
        filters: [{ name: 'SQLite database', extensions: ['db'] }],
      });
      if (!picked) return;
      await tauri.setDbPath(picked);
      dbPath = picked;
      dbStatus = 'Saved. Restart Postik for the change to take effect.';
    } catch (e) {
      dbStatus = `Failed: ${e}`;
    }
  }

  async function resetDbPath() {
    dbStatus = null;
    try {
      await tauri.setDbPath(null);
      dbPath = await tauri.currentDbPath();
      dbStatus = 'Reverted to default. Restart Postik for the change to take effect.';
    } catch (e) {
      dbStatus = `Failed: ${e}`;
    }
  }

  // Backup export / import. The user picks a path; we forward to Rust.
  let backupStatus = $state<string | null>(null);

  async function onExportBackup() {
    backupStatus = null;
    try {
      const today = new Date().toISOString().slice(0, 10);
      const path = await saveDialog({
        defaultPath: `postik-backup-${today}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });
      if (!path) return;
      await tauri.exportBackup(path);
      backupStatus = `Exported to ${path}`;
    } catch (e) {
      backupStatus = `Export failed: ${e}`;
    }
  }

  // AI organize: ask Claude to suggest tags + stack for every note,
  // show the count in a confirm dialog, apply on accept. The
  // suggestions are inspected by the user via the count + sample,
  // not blindly written — the prompt is intentionally narrow so
  // surprises are rare.
  let organizing = $state(false);
  let organizeStatus = $state<string | null>(null);
  async function onOrganizeNotes() {
    if (organizing) return;
    organizing = true;
    organizeStatus = null;
    try {
      const suggestions = await tauri.aiOrganizeNotes();
      if (suggestions.length === 0) {
        organizeStatus = 'No suggestions returned.';
        return;
      }
      const tagged = suggestions.filter((s) => s.tags).length;
      const stacked = suggestions.filter((s) => s.stack_id).length;
      const ok = await confirm(
        `Apply AI suggestions to ${suggestions.length} notes?\n• ${tagged} will get tags\n• ${stacked} will be assigned to a stack\n\nThis overwrites existing tags / stack assignment for those notes.`,
        { title: 'Organize notes', kind: 'info' },
      );
      if (!ok) {
        organizeStatus = 'Cancelled.';
        return;
      }
      const applied = await tauri.applyOrganizeSuggestions(suggestions);
      organizeStatus = `Applied to ${applied} notes.`;
      await notesStore.load();
    } catch (e) {
      organizeStatus = `Organize failed: ${e}`;
    } finally {
      organizing = false;
    }
  }

  async function onImportBackup() {
    backupStatus = null;
    const ok = await confirm(
      'Importing replaces every note in this Postik install with the backup. Continue?',
      { title: 'Import backup', kind: 'warning' },
    );
    if (!ok) return;
    try {
      const path = await openDialog({
        multiple: false,
        directory: false,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });
      if (!path || typeof path !== 'string') return;
      const count = await tauri.importBackup(path);
      backupStatus = `Imported ${count} notes`;
      // Refresh the controller's local note list so the UI reflects the
      // new state without a manual reload.
      await notesStore.load();
    } catch (e) {
      backupStatus = `Import failed: ${e}`;
    }
  }
</script>

<svelte:window onkeydown={onRecordKey} />

<div class="settings-view">
  <header class="header">
    <button class="back-btn" onclick={onBack} aria-label="Back to notes" title="Back">
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <path
          d="M10 3L5 8L10 13"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <h1>Settings</h1>
  </header>

  <div class="content">
    <section>
      <h2 class="section-heading">Privacy</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Hide from screen sharing</div>
          <div class="row-helper">
            Postik windows render as black rectangles in screen captures (Zoom, Meet, Teams,
            QuickTime). The tray icon stays visible — use ⌘⇧H to hide it too.
          </div>
        </div>
        <Switch
          checked={settingsStore.privacyHideFromCapture}
          onChange={(v) => settingsStore.setPrivacyHideFromCapture(v)}
          label="Hide from screen sharing"
        />
      </div>
    </section>

    <section>
      <h2 class="section-heading">Playback</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Sound on timer end</div>
          <div class="row-helper">
            Plays a chime that loops until you click Dismiss on the finished timer.
          </div>
        </div>
        <Switch
          checked={settingsStore.soundOnTimerEnd}
          onChange={(v) => settingsStore.setSoundOnTimerEnd(v)}
          label="Sound on timer end"
        />
      </div>

      <div class="row">
        <div class="row-text">
          <div class="row-label">Auto-start next pomodoro phase</div>
          <div class="row-helper">
            When on, pomodoro phases roll work → break → work without pausing. Off pauses at each
            boundary so you can acknowledge before the next phase begins.
          </div>
        </div>
        <Switch
          checked={settingsStore.pomodoroAutoStart}
          onChange={(v) => settingsStore.setPomodoroAutoStart(v)}
          label="Auto-start next pomodoro phase"
        />
      </div>

      {#if settingsStore.soundOnTimerEnd}
        <div class="sound-row">
          <div class="row-label sound-row-label">Chime</div>
          <div class="sound-list">
            {#each SOUND_CHOICES as choice (choice.id)}
              <button
                class="sound-option"
                class:selected={settingsStore.soundChoice === choice.id}
                onclick={() => selectSound(choice.id)}
                aria-pressed={settingsStore.soundChoice === choice.id}
              >
                <div class="sound-radio" aria-hidden="true">
                  {#if settingsStore.soundChoice === choice.id}
                    <div class="sound-radio-dot"></div>
                  {/if}
                </div>
                <div class="sound-text">
                  <div class="sound-label">{choice.label}</div>
                  <div class="sound-desc">{choice.description}</div>
                </div>
                <span class="sound-preview-hint" aria-hidden="true">▸</span>
              </button>
            {/each}
          </div>
          <div class="row-helper sound-helper">Click any option to preview and select.</div>
        </div>
      {/if}
    </section>

    <section>
      <h2 class="section-heading">Focus stats</h2>
      {#if stats}
        <div class="stats-row">
          <div class="stats-figure">
            <div class="stats-value">{fmtMinutes(stats.today_seconds)}</div>
            <div class="stats-caption">today</div>
          </div>
          <div class="stats-figure">
            <div class="stats-value">{fmtMinutes(stats.week_seconds)}</div>
            <div class="stats-caption">last 7 days</div>
          </div>
        </div>
        <div class="stats-chart">
          {#each stats.last_seven_days as b, i (b.date)}
            {@const last = i === stats.last_seven_days.length - 1}
            <div class="stats-bar">
              <div
                class="stats-bar-fill"
                style="height: {Math.max(2, (b.seconds / maxBucketSeconds) * 64)}px"
                title={`${dayLabel(b.date)}: ${fmtMinutes(b.seconds)}`}
              ></div>
              <div class="stats-bar-label" class:current={last}>
                {last ? 'Today' : dayLabel(b.date)}
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <p class="row-helper" style="padding: 0 16px 12px">Loading…</p>
      {/if}
    </section>

    <section>
      <h2 class="section-heading">Keyboard shortcuts</h2>
      <p class="row-helper" style="padding: 0 16px 8px">
        Click a binding to record a new keystroke. Hit Esc to cancel.
      </p>
      {#each shortcuts as s (s.action)}
        <div class="row shortcut-row">
          <div class="row-text">
            <div class="row-label">{shortcutLabel(s.action)}</div>
          </div>
          <button
            class="shortcut-btn"
            class:recording={recordingFor === s.action}
            onclick={() => (recordingFor = recordingFor === s.action ? null : s.action)}
          >
            {#if recordingFor === s.action}
              Press a key…
            {:else}
              {prettyAccelerator(s.accelerator)}
            {/if}
          </button>
          {#if s.accelerator !== s.default_accelerator}
            <button
              class="reset-btn"
              onclick={() => resetShortcut(s.action)}
              title="Reset to default"
              aria-label="Reset to default"
            >
              ↺
            </button>
          {/if}
        </div>
      {/each}
      {#if shortcutError}
        <p class="row-helper" style="padding: 0 16px 8px; color: var(--accent)">
          {shortcutError}
        </p>
      {/if}
    </section>

    <section>
      <h2 class="section-heading">Focus</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Blocked hosts during pomodoro</div>
          <div class="row-helper">
            One host per line (e.g. <code>youtube.com</code>). Postik prompts before opening a
            matching URL while a pomodoro work session is running. Acts on URLs Postik opens (note
            links, post-timer actions) — it can't block your browser directly.
          </div>
        </div>
      </div>
      <div class="row" style="padding-top: 0">
        <textarea
          class="hosts-area"
          placeholder="youtube.com&#10;reddit.com&#10;twitter.com"
          value={settingsStore.focusBlockedHosts}
          oninput={(e) =>
            settingsStore.setFocusBlockedHosts((e.currentTarget as HTMLTextAreaElement).value)}
          aria-label="Blocked hosts"
        ></textarea>
      </div>
    </section>

    <section>
      <h2 class="section-heading">Layout</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Sidebar mode</div>
          <div class="row-helper">
            Dock the controller as a thin column against the right edge of your screen. Notes still
            float as separate windows. Toggle off to revert to the default floating-controller
            layout.
          </div>
        </div>
        <Switch
          checked={settingsStore.sidebarMode}
          onChange={(v) => settingsStore.setSidebarMode(v)}
          label="Sidebar mode"
        />
      </div>
    </section>

    <section>
      <h2 class="section-heading">AI</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Anthropic API key</div>
          <div class="row-helper">
            Powers the per-note "Summarize" button. Get a key at console.anthropic.com. Stored
            locally, never sent anywhere except api.anthropic.com.
          </div>
        </div>
      </div>
      <div class="row" style="padding-top: 0">
        <input
          type="password"
          class="hosts-area"
          style="min-height: 32px; font-family: ui-monospace, monospace"
          placeholder="sk-ant-..."
          value={settingsStore.anthropicApiKey}
          oninput={(e) =>
            settingsStore.setAnthropicApiKey((e.currentTarget as HTMLInputElement).value)}
          aria-label="Anthropic API key"
        />
      </div>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Organize my notes</div>
          <div class="row-helper">
            One-shot: Claude reads your notes and suggests tags + stack assignments. You confirm
            before anything is written. Useful once you have 20+ notes.
          </div>
        </div>
        <button class="update-btn" onclick={onOrganizeNotes} disabled={organizing}>
          {organizing ? 'Organizing…' : 'Organize'}
        </button>
      </div>
      {#if organizeStatus}
        <div class="row" style="padding-top: 0">
          <p class="row-helper" style="padding: 0">{organizeStatus}</p>
        </div>
      {/if}
    </section>

    <section>
      <h2 class="section-heading">Storage</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Database location</div>
          <div class="row-helper">
            Pointing this at an iCloud / Dropbox folder gives you cross-device sync without a
            backend. Caveats: SQLite over network filesystems is fragile — keep one device writing
            at a time, expect possible conflicts. Restart Postik for changes to take effect.
          </div>
        </div>
      </div>
      <div class="row" style="padding-top: 0; flex-direction: column; align-items: stretch">
        <code class="db-path">{dbPath || 'Loading…'}</code>
        <div style="display: flex; gap: 6px">
          <button class="update-btn" onclick={pickDbPath}>Choose path…</button>
          <button class="update-btn" onclick={resetDbPath}>Reset to default</button>
        </div>
        {#if dbStatus}
          <p class="row-helper" style="padding: 0; margin-top: 6px">{dbStatus}</p>
        {/if}
      </div>
    </section>

    <section>
      <h2 class="section-heading">Backup</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Export all notes</div>
          <div class="row-helper">
            Saves a JSON snapshot with every note (active + archived), tags, recurring schedules,
            and settings. Google tokens and pomodoro history are not included.
          </div>
        </div>
        <button class="update-btn" onclick={onExportBackup}>Export…</button>
      </div>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Import backup</div>
          <div class="row-helper">
            Replaces every note with the snapshot. Settings are merged in.
          </div>
        </div>
        <button class="update-btn" onclick={onImportBackup}>Import…</button>
      </div>
      {#if backupStatus}
        <p class="row-helper" style="padding: 0 16px 12px">{backupStatus}</p>
      {/if}
    </section>

    <section>
      <h2 class="section-heading">Updates</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Check for updates</div>
          <div class="row-helper">
            Postik checks GitHub releases for a newer version. The download is signed; if
            verification fails, the install is rejected.
          </div>
        </div>
        {#if updateStatus.kind === 'idle' || updateStatus.kind === 'up-to-date' || updateStatus.kind === 'error'}
          <button class="update-btn" onclick={onCheckUpdate}>Check now</button>
        {:else if updateStatus.kind === 'checking'}
          <button class="update-btn" disabled>Checking…</button>
        {:else if updateStatus.kind === 'downloading'}
          <button class="update-btn" disabled>
            {updateStatus.total
              ? `${Math.round((updateStatus.downloaded / updateStatus.total) * 100)}%`
              : '…'}
          </button>
        {:else if updateStatus.kind === 'ready'}
          <button class="update-btn ready" onclick={onRestart}>Restart</button>
        {/if}
      </div>
      {#if updateStatus.kind === 'up-to-date'}
        <p class="row-helper" style="padding: 0 16px 12px">You're on the latest version.</p>
      {:else if updateStatus.kind === 'available'}
        <p class="row-helper" style="padding: 0 16px 12px">
          v{updateStatus.version} is available — downloading.
        </p>
      {:else if updateStatus.kind === 'ready'}
        <p class="row-helper" style="padding: 0 16px 12px">
          Update downloaded. Click Restart to apply.
        </p>
      {:else if updateStatus.kind === 'error'}
        <p class="row-helper" style="padding: 0 16px 12px; color: var(--accent)">
          {updateStatus.message}
        </p>
      {/if}
    </section>
  </div>
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .header {
    display: flex;
    align-items: center;
    height: 44px;
    padding: 0 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
    gap: 8px;
  }
  .header h1 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
    letter-spacing: -0.01em;
  }
  .back-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    color: var(--text-muted);
    transition: background-color 120ms ease-out;
  }
  .back-btn:hover {
    background: rgba(0, 0, 0, 0.05);
    color: inherit;
  }

  .content {
    flex: 1;
    overflow-y: auto;
  }

  section {
    padding-bottom: 8px;
  }
  .section-heading {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    margin: 0;
    padding: 16px 16px 8px;
  }

  .row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 10px 16px;
  }
  .row-text {
    flex: 1;
    min-width: 0;
  }
  .row-label {
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 4px;
  }
  .row-helper {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .sound-row {
    padding: 0 16px 12px;
  }
  .sound-row-label {
    margin-bottom: 6px;
  }
  .sound-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .sound-option {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid transparent;
    cursor: pointer;
    width: 100%;
    text-align: left;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out;
  }
  .sound-option:hover {
    background: rgba(216, 90, 48, 0.05);
  }
  .sound-option.selected {
    background: rgba(216, 90, 48, 0.08);
    border-color: rgba(216, 90, 48, 0.3);
  }
  .sound-radio {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 1.5px solid rgba(0, 0, 0, 0.25);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color 120ms ease-out;
  }
  .sound-option.selected .sound-radio {
    border-color: var(--accent);
  }
  .sound-radio-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
  }
  .sound-text {
    flex: 1;
    min-width: 0;
  }
  .sound-label {
    font-size: 12px;
    font-weight: 500;
  }
  .sound-desc {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: 1px;
  }
  .sound-preview-hint {
    font-size: 10px;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity 120ms ease-out;
  }
  .sound-option:hover .sound-preview-hint {
    opacity: 0.7;
  }
  .sound-helper {
    margin-top: 8px;
  }
  .stats-row {
    display: flex;
    gap: 16px;
    padding: 4px 16px 8px;
  }
  .stats-figure {
    flex: 1;
  }
  .stats-value {
    font-size: 18px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--accent);
  }
  .stats-caption {
    font-size: 10px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin-top: 2px;
  }
  .stats-chart {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    padding: 12px 16px 12px;
    height: 90px;
  }
  .stats-bar {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .stats-bar-fill {
    width: 100%;
    background: rgba(216, 90, 48, 0.6);
    border-radius: 2px 2px 0 0;
    min-height: 2px;
  }
  .stats-bar-label {
    font-size: 9px;
    color: var(--text-muted);
  }
  .stats-bar-label.current {
    color: var(--accent);
    font-weight: 600;
  }

  .shortcut-row {
    align-items: center;
    gap: 6px;
  }
  .shortcut-btn {
    flex-shrink: 0;
    padding: 5px 10px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.05);
    font-size: 11px;
    font-weight: 600;
    color: inherit;
    cursor: pointer;
    font-variant-numeric: tabular-nums;
    min-width: 84px;
    text-align: center;
  }
  .shortcut-btn:hover {
    background: rgba(216, 90, 48, 0.12);
    color: var(--accent);
  }
  .shortcut-btn.recording {
    background: var(--accent);
    color: white;
    animation: pulse 1.2s ease-in-out infinite;
  }
  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.7;
    }
  }
  .reset-btn {
    width: 22px;
    height: 22px;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
  }
  .reset-btn:hover {
    background: rgba(0, 0, 0, 0.05);
    color: inherit;
  }
  .hosts-area {
    width: 100%;
    min-height: 80px;
    padding: 8px 12px;
    margin: 0 16px;
    border-radius: 5px;
    border: 1px solid var(--border-subtle);
    background: rgba(0, 0, 0, 0.03);
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 11px;
    color: inherit;
    resize: vertical;
  }
  .hosts-area:focus {
    outline: none;
    border-color: rgba(216, 90, 48, 0.4);
    background: rgba(0, 0, 0, 0.01);
  }
  .db-path {
    display: block;
    margin: 0 16px 8px;
    padding: 6px 10px;
    background: rgba(0, 0, 0, 0.04);
    border-radius: 4px;
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 10px;
    word-break: break-all;
  }

  .update-btn {
    flex-shrink: 0;
    padding: 5px 12px;
    border-radius: 5px;
    background: rgba(0, 0, 0, 0.05);
    font-size: 11px;
    font-weight: 500;
    color: inherit;
    cursor: pointer;
    transition:
      background-color 120ms ease-out,
      color 120ms ease-out;
  }
  .update-btn:hover:not(:disabled) {
    background: rgba(216, 90, 48, 0.12);
    color: var(--accent);
  }
  .update-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .update-btn.ready {
    background: var(--accent);
    color: white;
  }
  .update-btn.ready:hover {
    background: #c64f29;
  }

  @media (prefers-color-scheme: dark) {
    .back-btn:hover {
      background: rgba(255, 255, 255, 0.08);
    }
    .sound-option {
      background: rgba(255, 255, 255, 0.04);
    }
    .sound-option:hover {
      background: rgba(216, 90, 48, 0.18);
    }
    .sound-option.selected {
      background: rgba(216, 90, 48, 0.22);
    }
    .sound-radio {
      border-color: rgba(255, 255, 255, 0.3);
    }
  }
</style>
