<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import TitleBar from './TitleBar.svelte';
  import Timer from './Timer.svelte';
  import AppearancePopover from './AppearancePopover.svelte';
  import { tauri } from '../utils/tauri';
  import { getColor, resolveTextColor } from '../utils/colors';
  import { startTimerDoneLoop, stopTimerDoneLoop } from '../utils/sound';
  import { normaliseUrl, toggleChecklistAt, urlAtPosition } from '../utils/textarea-actions';
  import { renderMarkdown } from '../utils/markdown';
  import { settingsStore } from '../stores/settings.svelte';
  import type {
    ColorId,
    NoteConfig,
    TextColorId,
    TimerStatePayload,
    TimerTickPayload,
    TimerDonePayload,
  } from '../types';

  interface Props {
    noteId: string;
  }

  let { noteId }: Props = $props();

  let _config = $state<NoteConfig | null>(null);
  // Notes backed by a Google Calendar event are read-only — content +
  // timer are controlled by the calendar sync, not the user.
  let isEvent = $derived(_config?.event_id != null);
  let content = $state('');
  let colorId = $state<ColorId>('amber');
  let textColorId = $state<TextColorId>('auto');
  let opacity = $state(1);
  let pinned = $state(true);
  let focusMode = $state(false);
  // Markdown preview vs raw-text editor. Toggled from the title bar.
  // Event-backed notes default to preview because they're already
  // read-only — markdown looks nicer than raw text for an event body.
  let previewMode = $state(false);
  let renderedHtml = $derived(previewMode ? renderMarkdown(content) : '');
  let timer = $state<TimerStatePayload | null>(null);
  let flashing = $state(false);

  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let cleanup: UnlistenFn[] = [];

  $effect(() => {
    const c = getColor(colorId);
    document.documentElement.style.setProperty('--note-fill', c.fill);
    document.documentElement.style.setProperty('--note-border', c.border);
    document.documentElement.style.setProperty(
      '--note-text',
      resolveTextColor(textColorId, c.text),
    );
  });

  // The chime loops while the timer is in 'done' state and stops the moment
  // the user dismisses (which transitions state away from 'done'). Respect
  // the user's "Sound on timer end" preference — disable the loop when off.
  $effect(() => {
    if (timer?.state === 'done' && settingsStore.soundOnTimerEnd) {
      startTimerDoneLoop(settingsStore.soundChoice);
    } else {
      stopTimerDoneLoop();
    }
  });

  onMount(async () => {
    settingsStore.load();
    const all = await tauri.listNotes();
    const found = all.find((n) => n.id === noteId);
    if (!found) {
      document.body.textContent = 'Note not found';
      return;
    }
    _config = found;
    content = found.content;
    colorId = found.color_id;
    textColorId = (found.text_color ?? 'auto') as TextColorId;
    opacity = found.opacity;
    pinned = found.always_on_top;

    timer = await tauri.getTimerState(noteId);

    cleanup.push(
      await listen<TimerTickPayload>('timer:tick', (e) => {
        if (e.payload.note_id !== noteId) return;
        timer = {
          note_id: e.payload.note_id,
          mode: e.payload.mode,
          state: e.payload.state,
          duration_seconds: timer?.duration_seconds ?? null,
          elapsed_seconds: e.payload.elapsed_seconds,
          remaining_seconds: e.payload.remaining_seconds,
          pomodoro_phase: e.payload.phase,
        };
      }),
      await listen<TimerDonePayload>('timer:done', (e) => {
        if (e.payload.note_id !== noteId) return;
        flashing = true;
        setTimeout(() => (flashing = false), 3000);
        // Pomodoro auto-rolls into the next phase, so it stays in 'running'
        // state — only countdowns end up "done" and need the dismiss UI.
        if (e.payload.mode === 'countdown' && timer) {
          timer = { ...timer, state: 'done', remaining_seconds: 0 };
        }
      }),
      // Backend cancels (Dismiss button, calendar bell toggle, auto-sync
      // pruning) emit `timer:cancelled` so this window can drop the chime
      // and the Done UI even when the cancel didn't originate here.
      await listen<{ note_id: string }>('timer:cancelled', (e) => {
        if (e.payload.note_id !== noteId) return;
        timer = null;
      }),
    );
  });

  onDestroy(() => {
    cleanup.forEach((fn) => fn());
    if (saveTimer) clearTimeout(saveTimer);
    stopTimerDoneLoop();
  });

  async function refreshTimer() {
    timer = await tauri.getTimerState(noteId);
  }

  function onContentInput() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      tauri.updateNoteContent(noteId, content).catch(console.error);
    }, 400);
  }

  // Click handler for the textarea body — handles two interactions:
  //
  // 1. ⌘/Ctrl + click on a URL → open in default browser (A2).
  // 2. Plain click on a `- [ ]` / `- [x]` bracket → toggle (A3).
  //
  // We intentionally only act on the bracket characters for (2), so
  // text-editing clicks elsewhere on a checklist line still place the
  // cursor as expected.
  async function onContentClick(e: MouseEvent) {
    if (isEvent) return; // event-backed notes are read-only
    const ta = e.currentTarget as HTMLTextAreaElement;
    const pos = ta.selectionStart;
    if (e.metaKey || e.ctrlKey) {
      const url = urlAtPosition(content, pos);
      if (url) {
        e.preventDefault();
        await tauri.openUrl(normaliseUrl(url));
        return;
      }
    }
    const result = toggleChecklistAt(content, pos);
    if (result) {
      e.preventDefault();
      content = result.content;
      // Persist immediately rather than wait for the input debounce —
      // there's no further keystroke coming and the user expects the
      // toggle to stick.
      await tauri.updateNoteContent(noteId, content);
      // Restore caret on the next paint.
      requestAnimationFrame(() => {
        ta.selectionStart = result.caret;
        ta.selectionEnd = result.caret;
      });
    }
  }

  async function changeColor(id: ColorId) {
    colorId = id;
    await tauri.updateNoteColor(noteId, id);
  }

  async function changeTextColor(id: TextColorId) {
    textColorId = id;
    await tauri.updateNoteTextColor(noteId, id === 'auto' ? null : id);
  }

  async function changeOpacity(v: number) {
    opacity = v;
    await tauri.updateNoteOpacity(noteId, v);
  }

  async function togglePin() {
    pinned = await tauri.toggleAlwaysOnTop(noteId);
  }

  function togglePreview() {
    previewMode = !previewMode;
  }

  /** Click handler for the rendered preview. Intercepts:
   *  - anchor clicks → opens via tauri.openUrl (the user wants the URL,
   *    not in-app navigation that would unload the note window).
   *  - checkbox clicks → finds the Nth `- [ ]` / `- [x]` line in source
   *    and toggles it. The order of <input type=checkbox> in the
   *    rendered HTML matches the source-line order one-to-one because
   *    that's how marked emits task lists.
   */
  async function onPreviewClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.tagName === 'A') {
      const href = (target as HTMLAnchorElement).getAttribute('href');
      if (href) {
        e.preventDefault();
        await tauri.openUrl(href);
      }
      return;
    }
    if (
      target.tagName === 'INPUT' &&
      (target as HTMLInputElement).type === 'checkbox' &&
      !isEvent
    ) {
      e.preventDefault();
      const root = e.currentTarget as HTMLElement;
      const all = root.querySelectorAll<HTMLInputElement>('input[type="checkbox"]');
      const idx = Array.from(all).indexOf(target as HTMLInputElement);
      if (idx === -1) return;
      const lines = content.split('\n');
      let count = -1;
      for (let i = 0; i < lines.length; i++) {
        if (/^\s*[-*]\s\[( |x|X)\]/.test(lines[i])) {
          count++;
          if (count === idx) {
            lines[i] = lines[i].replace(/\[( |x|X)\]/, (_m, p) => (p === ' ' ? '[x]' : '[ ]'));
            content = lines.join('\n');
            await tauri.updateNoteContent(noteId, content);
            return;
          }
        }
      }
    }
  }

  async function toggleFocus() {
    if (focusMode) {
      // Exit focus mode → restore every other note window.
      await tauri.showAllNotes();
      focusMode = false;
    } else {
      await tauri.focusOnlyNote(noteId);
      focusMode = true;
    }
  }

  async function closeNote() {
    // Closing the note window is non-destructive: the content stays in
    // SQLite and the note remains in the controller list. Permanent
    // deletion happens via the trash icon in the controller, which has
    // its own confirmation. So no dialog needed here.
    await getCurrentWindow().hide();
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'w') {
      e.preventDefault();
      closeNote();
    }
  }}
/>

<div
  class="note-shell"
  class:transparent={colorId === 'transparent'}
  style="--note-opacity: {opacity};"
>
  <TitleBar
    {colorId}
    {pinned}
    {focusMode}
    {previewMode}
    onTogglePin={togglePin}
    onToggleFocus={toggleFocus}
    onTogglePreview={togglePreview}
    onOpenTimer={() => {
      const el = document.querySelector<HTMLButtonElement>('.timer-bar button');
      el?.click();
    }}
    onClose={closeNote}
  />

  {#if previewMode}
    <!-- The click handler delegates to interactive children (anchors and
         checkboxes), which carry their own accessibility semantics. -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="content preview" role="document" aria-label="Note preview" onclick={onPreviewClick}>
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html renderedHtml}
    </div>
  {:else}
    <textarea
      class="content"
      bind:value={content}
      oninput={onContentInput}
      onclick={onContentClick}
      placeholder={isEvent ? '' : 'Start typing…'}
      aria-label={isEvent ? 'Calendar event (read-only)' : 'Note content'}
      readonly={isEvent}
    ></textarea>
  {/if}

  <div class="bottom-bar">
    <div class="timer-wrap">
      <Timer {noteId} {timer} {flashing} onChange={refreshTimer} />
    </div>
    <div class="micro-controls">
      <AppearancePopover
        {colorId}
        {opacity}
        {textColorId}
        onColorChange={changeColor}
        onOpacityChange={changeOpacity}
        onTextColorChange={changeTextColor}
      />
    </div>
  </div>
</div>

<style>
  :global(html, body) {
    background: transparent;
    overflow: hidden;
  }

  .note-shell {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    /* `--note-opacity` is set inline from the persisted value; the
     * effective alpha resolves to that by default, but the :hover rule
     * below bumps it to 1 so faded notes become readable while
     * interacting and snap back when the cursor leaves. */
    --note-opacity-effective: var(--note-opacity, 1);
    background-color: color-mix(
      in srgb,
      var(--note-fill) calc(var(--note-opacity-effective) * 100%),
      transparent
    );
    border: 0.5px solid var(--note-border);
    border-radius: 8px;
    color: var(--note-text);
    overflow: hidden;
    transition: background-color 140ms ease-out;
  }
  .note-shell:hover {
    --note-opacity-effective: 1;
  }
  .note-shell.transparent {
    background: rgba(255, 255, 255, 0.05);
  }
  .content {
    flex: 1;
    border: none;
    background: transparent;
    padding: 8px 12px;
    font-size: 13px;
    line-height: 1.5;
    color: inherit;
    outline: none;
  }
  .content.preview {
    overflow-y: auto;
    cursor: default;
  }
  .content.preview :global(p) {
    margin: 0 0 6px;
  }
  .content.preview :global(h1),
  .content.preview :global(h2),
  .content.preview :global(h3) {
    margin: 8px 0 4px;
    font-size: 13px;
    font-weight: 700;
  }
  .content.preview :global(ul),
  .content.preview :global(ol) {
    margin: 0 0 6px;
    padding-left: 20px;
  }
  .content.preview :global(li) {
    margin: 2px 0;
  }
  .content.preview :global(a) {
    color: var(--accent);
    text-decoration: underline;
    text-decoration-thickness: 0.5px;
    cursor: pointer;
  }
  .content.preview :global(code) {
    background: rgba(0, 0, 0, 0.06);
    padding: 0 4px;
    border-radius: 3px;
    font-size: 11px;
    font-family: ui-monospace, SFMono-Regular, monospace;
  }
  .content.preview :global(pre) {
    background: rgba(0, 0, 0, 0.05);
    padding: 6px 8px;
    border-radius: 4px;
    overflow-x: auto;
    margin: 0 0 6px;
  }
  .content.preview :global(pre code) {
    background: none;
    padding: 0;
  }
  .content.preview :global(blockquote) {
    border-left: 2px solid var(--note-border, currentColor);
    padding-left: 8px;
    margin: 0 0 6px;
    opacity: 0.85;
  }
  .content.preview :global(input[type='checkbox']) {
    cursor: pointer;
    accent-color: var(--accent);
    margin-right: 4px;
  }
  .content.preview :global(li:has(> input[type='checkbox'])) {
    list-style: none;
    margin-left: -16px;
  }
  .content.preview :global(hr) {
    border: 0;
    border-top: 1px solid rgba(0, 0, 0, 0.08);
    margin: 8px 0;
  }
  .bottom-bar {
    display: flex;
    align-items: stretch;
    flex-shrink: 0;
    border-top: 1px solid rgba(0, 0, 0, 0.06);
  }
  .timer-wrap {
    flex: 1;
    min-width: 0;
  }
  /* Timer renders its own border-top; suppress the duplicate now that
     bottom-bar provides the divider. */
  .timer-wrap :global(.timer-bar) {
    border-top: none;
  }
  .micro-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 8px;
    flex-shrink: 0;
  }
</style>
