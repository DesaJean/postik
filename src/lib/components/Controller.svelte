<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { notesStore } from '../stores/notes.svelte';
  import { tauri } from '../utils/tauri';
  import { getColor } from '../utils/colors';
  import type { TimerTickPayload } from '../types';
  import { formatDuration } from '../utils/time-parser';

  let timersByNote = $state<Record<string, TimerTickPayload>>({});
  let showSettings = $state(false);

  onMount(() => {
    notesStore.load();

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

<header class="controller-header">
  <div class="title">Postik</div>
  <button class="primary" onclick={newNote} aria-label="Create new note"> + New note </button>
</header>

<main class="controller-main">
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
            style="--c: {color.fill}; --c-border: {color.border};"
            onclick={() => focusNote(note.id)}
            aria-label={`Open note: ${note.content.slice(0, 40) || '(empty)'}`}
          >
            <span class="dot" aria-hidden="true"></span>
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

<footer class="controller-footer">
  <button
    class="icon-button"
    onclick={() => (showSettings = !showSettings)}
    aria-label="Settings"
    title="Settings"
  >
    ⚙
  </button>
</footer>

{#if showSettings}
  <div class="modal-backdrop" onclick={() => (showSettings = false)} role="presentation">
    <div
      class="modal"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.key === 'Escape' && (showSettings = false)}
      role="dialog"
      tabindex="-1"
      aria-label="Settings"
    >
      <h2>Settings</h2>
      <p>Coming in v0.2</p>
      <button class="primary" onclick={() => (showSettings = false)}>Close</button>
    </div>
  </div>
{/if}

<style>
  .controller-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-subtle);
  }
  .title {
    font-weight: 600;
    font-size: 14px;
  }
  .primary {
    background: var(--accent);
    color: white;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 500;
  }
  .primary:hover {
    background: #c64f29;
  }

  .controller-main {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .empty-state {
    text-align: center;
    padding: 48px 16px;
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
    gap: 4px;
  }
  .note-row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    background: var(--c);
    border: 1px solid var(--c-border);
    border-radius: 6px;
    text-align: left;
  }
  .note-row:hover {
    filter: brightness(0.97);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--c-border);
    flex-shrink: 0;
  }
  .preview {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #1c1c1a;
  }
  .timer-badge {
    font-size: 11px;
    color: #1c1c1a;
    background: rgba(255, 255, 255, 0.6);
    padding: 2px 6px;
    border-radius: 3px;
  }

  .controller-footer {
    padding: 8px 12px;
    display: flex;
    justify-content: flex-end;
    border-top: 1px solid var(--border-subtle);
  }
  .icon-button {
    width: 28px;
    height: 28px;
    border-radius: 4px;
    font-size: 14px;
    color: var(--text-muted);
  }
  .icon-button:hover {
    background: rgba(0, 0, 0, 0.06);
    color: inherit;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: var(--bg-controller);
    padding: 24px;
    border-radius: 8px;
    min-width: 280px;
  }
  .modal h2 {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 600;
  }
  .modal p {
    color: var(--text-muted);
    margin: 0 0 16px;
  }

  @media (prefers-color-scheme: dark) {
    .modal {
      background: var(--bg-controller-dark);
    }
    kbd {
      background: rgba(255, 255, 255, 0.1);
    }
    .icon-button:hover {
      background: rgba(255, 255, 255, 0.08);
    }
  }
</style>
