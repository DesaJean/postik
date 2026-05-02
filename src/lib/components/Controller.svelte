<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import { notesStore, NOTE_TEMPLATES } from '../stores/notes.svelte';
  import type { NoteTemplate } from '../stores/notes.svelte';
  import { distinctTags, parseTags } from '../utils/tags';
  import { settingsStore } from '../stores/settings.svelte';
  import { stacksStore } from '../stores/stacks.svelte';
  import { tauri } from '../utils/tauri';
  import { getColor } from '../utils/colors';
  import type { TimerTickPayload } from '../types';
  import { formatDuration } from '../utils/time-parser';
  import Settings from './Settings.svelte';
  import Calendar from './Calendar.svelte';

  let timersByNote = $state<Record<string, TimerTickPayload>>({});
  let view = $state<'notes' | 'settings'>('notes');
  let tab = $state<'notes' | 'calendar'>('notes');
  // Sub-mode within Notes tab: 'active' shows the working list, 'archived'
  // shows soft-deleted notes with Restore + Delete-forever actions.
  let notesView = $state<'active' | 'archived'>('active');

  // Search across notes. Filters by content (case-insensitive substring).
  // Empty string disables filtering; the controller falls back to the
  // full list. ⌘K / Ctrl+K focuses the search input from anywhere.
  let search = $state('');
  let searchInput: HTMLInputElement | null = $state(null);

  // Tag filter: when set, the list narrows to notes with that tag.
  // Cleared when the user clicks the same chip again or the × button.
  let tagFilter = $state<string | null>(null);
  let allTags = $derived(distinctTags(notesStore.notes));

  // Stack filter: 'all' (default) shows everything; a stack id narrows
  // the list to that stack; 'none' shows only unstacked notes. The
  // chip row is only rendered when at least one stack exists.
  let stackFilter = $state<string | 'all' | 'none'>('all');

  let filteredNotes = $derived.by(() => {
    const q = search.trim().toLowerCase();
    let list = notesStore.notes;
    if (stackFilter === 'none') {
      list = list.filter((n) => !n.stack_id);
    } else if (stackFilter !== 'all') {
      list = list.filter((n) => n.stack_id === stackFilter);
    }
    if (tagFilter) {
      list = list.filter((n) => parseTags(n.tags).includes(tagFilter!));
    }
    if (!q) return list;
    return list.filter((n) => n.content.toLowerCase().includes(q));
  });

  onMount(() => {
    notesStore.load();
    stacksStore.load();
    settingsStore.load();

    const unlistenTick = listen<TimerTickPayload>('timer:tick', (event) => {
      timersByNote = { ...timersByNote, [event.payload.note_id]: event.payload };
    });
    // The Rust shortcut handler already created the note via WindowManager;
    // we just refresh the list so the new entry shows up here. Calling
    // notesStore.create() would create a SECOND note.
    const unlistenShortcut = listen('shortcut:new-note', () => {
      notesStore.load();
    });

    return async () => {
      (await unlistenTick)();
      (await unlistenShortcut)();
    };
  });

  // ⌘K / Ctrl+K from anywhere in the controller focuses the search input
  // and switches to the Notes tab if the user is currently on Calendar.
  function onWindowKeyDown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      tab = 'notes';
      view = 'notes';
      // Defer focus to next tick so the input has rendered after the tab swap.
      queueMicrotask(() => searchInput?.focus());
    }
  }

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

  async function newFromTemplate(id: NoteTemplate['id']) {
    await notesStore.createFromTemplate(id);
  }

  // Drag-to-reorder. We only enable it when search is empty: filtering +
  // reordering at the same time would yield surprising indices because
  // the user-visible order doesn't match the underlying notesStore.notes
  // array. Search clears on Escape so this is a soft constraint.
  let dragFrom = $state<number | null>(null);
  let dragOver = $state<number | null>(null);

  function onDragStart(idx: number, e: DragEvent) {
    if (search.trim()) return;
    dragFrom = idx;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      // Firefox needs some payload to start the drag.
      e.dataTransfer.setData('text/plain', String(idx));
    }
  }
  function onDragOver(idx: number, e: DragEvent) {
    if (dragFrom === null) return;
    e.preventDefault();
    dragOver = idx;
  }
  function onDragEnd() {
    dragFrom = null;
    dragOver = null;
  }
  async function onDrop(idx: number, e: DragEvent) {
    e.preventDefault();
    const from = dragFrom;
    dragFrom = null;
    dragOver = null;
    if (from === null || from === idx) return;
    await notesStore.reorder(from, idx);
  }

  // The trash icon now archives (reversible). Permanent deletion is
  // only available from the archive view, with a confirmation.
  async function archiveNote(id: string) {
    await notesStore.archive(id);
  }

  async function unarchiveNote(id: string) {
    await notesStore.unarchive(id);
  }

  async function deleteForever(id: string, hasContent: boolean) {
    if (hasContent) {
      const ok = await confirm('Permanently delete this note? This cannot be undone.', {
        title: 'Delete forever',
        kind: 'warning',
      });
      if (!ok) return;
    }
    await notesStore.deletePermanently(id);
  }

  async function openArchive() {
    notesView = 'archived';
    await notesStore.loadArchived();
  }
</script>

<svelte:window onkeydown={onWindowKeyDown} />

<div class="shell">
  <div class="slider" class:show-settings={view === 'settings'}>
    <!-- Notes view -->
    <section class="pane">
      <header class="header">
        <div class="tabs">
          <button
            class="tab"
            class:active={tab === 'notes'}
            onclick={() => (tab = 'notes')}
            aria-pressed={tab === 'notes'}
          >
            Notes
          </button>
          <button
            class="tab"
            class:active={tab === 'calendar'}
            onclick={() => (tab = 'calendar')}
            aria-pressed={tab === 'calendar'}
          >
            Calendar
          </button>
        </div>
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

      {#if tab === 'notes' && notesView === 'archived'}
        <div class="archive-bar">
          <button
            class="archive-back"
            onclick={() => (notesView = 'active')}
            aria-label="Back to active notes"
          >
            ← Back
          </button>
          <span class="archive-title">Archived</span>
        </div>
        <main class="list-wrap">
          {#if notesStore.archived.length === 0}
            <div class="empty-state">
              <p>No archived notes.</p>
              <p class="muted">Notes you archive will appear here.</p>
            </div>
          {:else}
            <ul class="note-list">
              {#each notesStore.archived as note (note.id)}
                {@const color = getColor(note.color_id)}
                <li class="note-item">
                  <div class="note-row archived-row">
                    <span class="dot" style="background: {color.border};" aria-hidden="true"></span>
                    <span class="preview">
                      {note.content.slice(0, 40) || '(empty note)'}
                    </span>
                  </div>
                  <button
                    class="archive-action"
                    onclick={() => unarchiveNote(note.id)}
                    title="Restore"
                    aria-label="Restore note"
                  >
                    Restore
                  </button>
                  <button
                    class="archive-action danger"
                    onclick={() => deleteForever(note.id, note.content.trim().length > 0)}
                    title="Delete forever"
                    aria-label="Delete forever"
                  >
                    Delete
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </main>
      {:else if tab === 'notes'}
        <div class="cta-row">
          <button class="new-note-btn" onclick={newNote} aria-label="Create new note">
            <span class="plus">+</span>
            <span>New note</span>
          </button>
          <div class="template-chips">
            {#each NOTE_TEMPLATES.filter((t) => t.id !== 'blank') as tpl (tpl.id)}
              <button
                class="template-chip"
                onclick={() => newFromTemplate(tpl.id)}
                title={`New ${tpl.label.toLowerCase()} note`}
              >
                <span aria-hidden="true">{tpl.emoji}</span>
                <span>{tpl.label}</span>
              </button>
            {/each}
          </div>
        </div>

        {#if stacksStore.stacks.length > 0}
          <div class="stack-filter-row">
            <button
              class="stack-chip"
              class:active={stackFilter === 'all'}
              onclick={() => (stackFilter = 'all')}
              title="All notes"
            >
              All
            </button>
            {#each stacksStore.stacks as s (s.id)}
              <button
                class="stack-chip"
                class:active={stackFilter === s.id}
                onclick={() => (stackFilter = s.id)}
                style={s.color ? `--chip-accent: ${s.color}` : undefined}
                title={s.name}
              >
                <span class="stack-dot" aria-hidden="true"></span>
                {s.name}
              </button>
            {/each}
            <button
              class="stack-chip"
              class:active={stackFilter === 'none'}
              onclick={() => (stackFilter = 'none')}
              title="Notes without a stack"
            >
              Unstacked
            </button>
          </div>
        {/if}

        {#if allTags.length > 0}
          <div class="tag-filter-row">
            {#each allTags as t (t)}
              <button
                class="tag-filter-chip"
                class:active={tagFilter === t}
                onclick={() => (tagFilter = tagFilter === t ? null : t)}
              >
                #{t}
              </button>
            {/each}
            {#if tagFilter}
              <button
                class="tag-filter-clear"
                onclick={() => (tagFilter = null)}
                aria-label="Clear tag filter">×</button
              >
            {/if}
          </div>
        {/if}

        <div class="search-row">
          <svg class="search-icon" viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
            <circle cx="7" cy="7" r="4.5" fill="none" stroke="currentColor" stroke-width="1.4" />
            <path
              d="M10.5 10.5L13.5 13.5"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
          </svg>
          <input
            type="text"
            class="search-input"
            placeholder="Search notes…"
            bind:value={search}
            bind:this={searchInput}
            aria-label="Search notes"
          />
          {#if search}
            <button
              class="search-clear"
              onclick={() => (search = '')}
              aria-label="Clear search"
              title="Clear">×</button
            >
          {/if}
        </div>

        <main class="list-wrap">
          {#if notesStore.loading}
            <div class="empty-state">Loading…</div>
          {:else if notesStore.notes.length === 0}
            <div class="empty-state">
              <p>No notes yet.</p>
              <p class="muted">Press <kbd>⌘⇧N</kbd> or click "New note".</p>
            </div>
          {:else if filteredNotes.length === 0}
            <div class="empty-state">
              <p>No matches.</p>
              <p class="muted">Nothing for "{search.trim()}".</p>
            </div>
          {:else}
            <ul class="note-list">
              {#each filteredNotes as note, idx (note.id)}
                {@const color = getColor(note.color_id)}
                <li
                  class="note-item"
                  class:drag-over={dragOver === idx && dragFrom !== idx}
                  class:dragging={dragFrom === idx}
                  draggable={!search.trim()}
                  ondragstart={(e) => onDragStart(idx, e)}
                  ondragover={(e) => onDragOver(idx, e)}
                  ondrop={(e) => onDrop(idx, e)}
                  ondragend={onDragEnd}
                >
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
                  <button
                    class="delete-btn"
                    onclick={(e) => {
                      e.stopPropagation();
                      archiveNote(note.id);
                    }}
                    aria-label="Archive note"
                    title="Archive (recoverable)"
                  >
                    <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
                      <path
                        d="M3 4h10M6.5 4V2.5a1 1 0 0 1 1-1h1a1 1 0 0 1 1 1V4M5 4l.7 9a1 1 0 0 0 1 .9h2.6a1 1 0 0 0 1-.9L11 4M7 7v4M9 7v4"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      />
                    </svg>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </main>
        <div class="archive-footer">
          <button class="archive-link" onclick={openArchive}>View archived</button>
        </div>
      {:else}
        <Calendar />
      {/if}
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
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tab {
    padding: 4px 10px;
    border-radius: 6px;
    background: transparent;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background-color 120ms ease-out,
      color 120ms ease-out;
  }
  .tab:hover {
    background: rgba(0, 0, 0, 0.04);
    color: inherit;
  }
  .tab.active {
    background: rgba(216, 90, 48, 0.1);
    color: var(--accent);
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

  .template-chips {
    display: flex;
    gap: 4px;
    margin-top: 6px;
  }
  .template-chip {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    height: 24px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.04);
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    cursor: pointer;
    transition:
      background-color 120ms ease-out,
      color 120ms ease-out;
  }
  .template-chip:hover {
    background: rgba(216, 90, 48, 0.08);
    color: var(--accent);
  }

  .archive-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .archive-back {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    color: var(--text-muted);
    cursor: pointer;
  }
  .archive-back:hover {
    background: rgba(0, 0, 0, 0.05);
    color: inherit;
  }
  .archive-title {
    font-size: 12px;
    font-weight: 600;
  }
  .archived-row {
    flex: 1;
    pointer-events: none;
    opacity: 0.7;
  }
  .archive-action {
    padding: 3px 8px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.05);
    font-size: 10px;
    font-weight: 500;
    margin-right: 4px;
    cursor: pointer;
  }
  .archive-action:hover {
    background: rgba(216, 90, 48, 0.1);
    color: var(--accent);
  }
  .archive-action.danger:hover {
    background: rgba(216, 90, 48, 0.18);
    color: var(--accent);
  }
  .archive-footer {
    padding: 6px 12px 10px;
    flex-shrink: 0;
    text-align: center;
  }
  .archive-link {
    font-size: 10px;
    color: var(--text-muted);
    cursor: pointer;
  }
  .archive-link:hover {
    color: var(--accent);
    text-decoration: underline;
  }

  .stack-filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 0 12px 6px;
    align-items: center;
  }
  .stack-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 9px;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.05);
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    cursor: pointer;
  }
  .stack-chip:hover {
    background: rgba(216, 90, 48, 0.1);
    color: var(--accent);
  }
  .stack-chip.active {
    background: var(--chip-accent, var(--accent));
    color: white;
  }
  .stack-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--chip-accent, currentColor);
  }
  .stack-chip.active .stack-dot {
    background: rgba(255, 255, 255, 0.85);
  }

  .tag-filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 0 12px 6px;
    align-items: center;
  }
  .tag-filter-chip {
    padding: 2px 8px;
    border-radius: 10px;
    background: rgba(0, 0, 0, 0.05);
    font-size: 10px;
    font-weight: 500;
    color: var(--text-muted);
    cursor: pointer;
  }
  .tag-filter-chip:hover {
    background: rgba(216, 90, 48, 0.1);
    color: var(--accent);
  }
  .tag-filter-chip.active {
    background: var(--accent);
    color: white;
  }
  .tag-filter-clear {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    color: var(--text-muted);
    cursor: pointer;
  }
  .tag-filter-clear:hover {
    background: rgba(0, 0, 0, 0.05);
    color: inherit;
  }

  .search-row {
    position: relative;
    display: flex;
    align-items: center;
    margin: 0 12px 8px;
  }
  .search-icon {
    position: absolute;
    left: 9px;
    color: var(--text-muted);
    pointer-events: none;
  }
  .search-input {
    width: 100%;
    height: 28px;
    border-radius: 5px;
    border: 1px solid var(--border-subtle);
    background: rgba(0, 0, 0, 0.03);
    padding: 0 28px 0 28px;
    font-size: 12px;
    color: inherit;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out;
  }
  .search-input:focus {
    outline: none;
    background: rgba(0, 0, 0, 0.01);
    border-color: rgba(216, 90, 48, 0.4);
  }
  .search-clear {
    position: absolute;
    right: 4px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    color: var(--text-muted);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
  }
  .search-clear:hover {
    background: rgba(0, 0, 0, 0.05);
    color: inherit;
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
  .note-item {
    position: relative;
    transition: opacity 120ms ease-out;
  }
  .note-item.dragging {
    opacity: 0.4;
  }
  .note-item.drag-over::before {
    content: '';
    position: absolute;
    top: -2px;
    left: 4px;
    right: 4px;
    height: 2px;
    background: var(--accent);
    border-radius: 1px;
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
  .note-row:hover,
  .note-item:has(.delete-btn:hover) .note-row {
    background: rgba(0, 0, 0, 0.04);
  }

  .delete-btn {
    position: absolute;
    top: 50%;
    right: 8px;
    transform: translateY(-50%);
    width: 24px;
    height: 24px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    background: transparent;
    cursor: pointer;
    opacity: 0;
    pointer-events: none;
    transition:
      opacity 120ms ease-out,
      background-color 120ms ease-out,
      color 120ms ease-out;
  }
  .note-item:hover .delete-btn,
  .delete-btn:focus-visible {
    opacity: 1;
    pointer-events: auto;
  }
  .delete-btn:hover {
    background: rgba(216, 90, 48, 0.12);
    color: var(--accent);
  }
  /* Fade the timer badge when the row is hovered so it doesn't fight the
     delete button for the same slot. */
  .note-item:hover .timer-badge {
    opacity: 0;
  }
  .timer-badge {
    transition: opacity 120ms ease-out;
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
    .search-input {
      background: rgba(255, 255, 255, 0.05);
      border-color: var(--border-subtle-dark);
    }
    .search-input:focus {
      background: rgba(255, 255, 255, 0.08);
    }
    .search-clear:hover {
      background: rgba(255, 255, 255, 0.1);
    }
    .template-chip {
      background: rgba(255, 255, 255, 0.05);
    }
    .template-chip:hover {
      background: rgba(216, 90, 48, 0.18);
    }
  }
</style>
