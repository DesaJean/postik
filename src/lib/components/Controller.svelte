<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { notesStore } from '../stores/notes.svelte';
  import { settingsStore } from '../stores/settings.svelte';
  import { tauri } from '../utils/tauri';
  import { getColor } from '../utils/colors';
  import type { TimerTickPayload } from '../types';
  import { formatDuration } from '../utils/time-parser';
  import Settings from './Settings.svelte';

  let timersByNote = $state<Record<string, TimerTickPayload>>({});
  let view = $state<'notes' | 'settings'>('notes');

  onMount(() => {
    notesStore.load();
    settingsStore.load();

    const unlistenTick = listen<TimerTickPayload>('timer:tick', (event) => {
      timersByNote = { ...timersByNote, [event.payload.note_id]: event.payload };
    });
    const unlistenShortcut = listen('shortcut:new-note', () => {
      notesStore.create();
    });

    return async () => {
      (await unlistenTick)();
      (await unlistenShortcut)();
    };
  });

  function timerLabel(noteId: string): string | null {
    const t = timersByNote[noteId];
    if (!t || t.state !== 'running') return null;
    const display =
      t.remaining_seconds !== null
        ? formatDuration(t.remaining_seconds)
        : formatDuration(t.elapsed_seconds);
    return `⏱ ${display}`;
  }

  async function focusNote(id: string) {
    await tauri.focusNote(id);
  }

  async function newNote() {
    await notesStore.create();
  }
</script>

<div class="shell">
  <div class="slider" class:show-settings={view === 'settings'}>
    <!-- Notes view -->
    <section class="pane">
      <header class="header">
        <h1>Postik</h1>
        <button
          class="header-btn"
          onclick={() => (view = 'settings')}
          aria-label="Open settings"
          title="Settings"
        >
          <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
            <circle cx="8" cy="8" r="2.5" fill="none" stroke="currentColor" stroke-width="1.4" />
            <path
              d="M8 1.5v1.8M8 12.7v1.8M3.4 3.4l1.3 1.3M11.3 11.3l1.3 1.3M1.5 8h1.8M12.7 8h1.8M3.4 12.6l1.3-1.3M11.3 4.7l1.3-1.3"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
              fill="none"
            />
          </svg>
        </button>
      </header>

      <div class="cta-row">
        <button class="new-note-btn" onclick={newNote} aria-label="Create new note">
          <span class="plus">+</span>
          <span>New note</span>
        </button>
      </div>

      <main class="list-wrap">
        {#if notesStore.loading}
          <div class="empty-state">Loading…</div>
        {:else if notesStore.notes.length === 0}
          <div class="empty-state">
            <p>No notes yet.</p>
            <p class="muted">Press <kbd>⌘⇧N</kbd> or click "New note".</p>
          </div>
        {:else}
          <ul class="note-list">
            {#each notesStore.notes as note (note.id)}
              {@const color = getColor(note.color_id)}
              <li>
                <button
                  class="note-row"
                  onclick={() => focusNote(note.id)}
                  aria-label={`Open note: ${note.content.slice(0, 40) || '(empty)'}`}
                >
                  <span class="dot" style="background: {color.border};" aria-hidden="true"></span>
                  <span class="preview">
                    {note.content.slice(0, 40) || '(empty note)'}
                  </span>
                  {#if timerLabel(note.id)}
                    <span class="timer-badge">{timerLabel(note.id)}</span>
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </main>
    </section>

    <!-- Settings view -->
    <section class="pane">
      <Settings onBack={() => (view = 'notes')} />
    </section>
  </div>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .slider {
    display: flex;
    width: 200%;
    height: 100%;
    transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1);
    transform: translateX(0);
  }
  .slider.show-settings {
    transform: translateX(-50%);
  }
  .pane {
    width: 50%;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 44px;
    padding: 0 12px 0 16px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .header h1 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
    letter-spacing: -0.01em;
  }
  .header-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    color: var(--text-muted);
    transition:
      background-color 120ms ease-out,
      color 120ms ease-out;
  }
  .header-btn:hover {
    background: rgba(0, 0, 0, 0.05);
    color: inherit;
  }

  .cta-row {
    padding: 12px 12px 8px;
    flex-shrink: 0;
  }
  .new-note-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 100%;
    height: 34px;
    border-radius: 6px;
    border: 1px solid var(--border-subtle);
    background: transparent;
    font-size: 12px;
    font-weight: 500;
    color: inherit;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out;
  }
  .new-note-btn:hover {
    background: rgba(216, 90, 48, 0.06);
    border-color: rgba(216, 90, 48, 0.3);
    color: var(--accent);
  }
  .plus {
    font-size: 14px;
    font-weight: 400;
    line-height: 1;
  }

  .list-wrap {
    flex: 1;
    overflow-y: auto;
    padding: 0 8px 12px;
    min-height: 0;
  }

  .empty-state {
    text-align: center;
    padding: 32px 16px;
    color: var(--text-muted);
  }
  .empty-state .muted {
    font-size: 11px;
    margin-top: 8px;
  }
  kbd {
    background: rgba(0, 0, 0, 0.06);
    padding: 2px 6px;
    border-radius: 3px;
    font-family: var(--font-system);
    font-size: 11px;
  }

  .note-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .note-row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 10px;
    border-radius: 6px;
    text-align: left;
    color: inherit;
    transition: background-color 120ms ease-out;
  }
  .note-row:hover {
    background: rgba(0, 0, 0, 0.04);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .preview {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
    font-size: 13px;
  }
  .timer-badge {
    font-size: 11px;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
    background: rgba(0, 0, 0, 0.04);
    padding: 2px 6px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  @media (prefers-color-scheme: dark) {
    kbd {
      background: rgba(255, 255, 255, 0.1);
    }
    .header-btn:hover {
      background: rgba(255, 255, 255, 0.06);
    }
    .note-row:hover {
      background: rgba(255, 255, 255, 0.05);
    }
    .new-note-btn {
      border-color: var(--border-subtle-dark);
    }
    .timer-badge {
      background: rgba(255, 255, 255, 0.06);
    }
  }
</style>
