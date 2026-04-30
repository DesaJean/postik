<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { confirm } from '@tauri-apps/plugin-dialog';
  import TitleBar from './TitleBar.svelte';
  import Timer from './Timer.svelte';
  import ColorPicker from './ColorPicker.svelte';
  import OpacitySlider from './OpacitySlider.svelte';
  import { tauri } from '../utils/tauri';
  import { getColor } from '../utils/colors';
  import type {
    ColorId,
    NoteConfig,
    TimerStatePayload,
    TimerTickPayload,
    TimerDonePayload,
  } from '../types';

  interface Props {
    noteId: string;
  }

  let { noteId }: Props = $props();

  let _config = $state<NoteConfig | null>(null);
  let content = $state('');
  let colorId = $state<ColorId>('amber');
  let opacity = $state(1);
  let pinned = $state(true);
  let timer = $state<TimerStatePayload | null>(null);
  let flashing = $state(false);

  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let cleanup: UnlistenFn[] = [];

  $effect(() => {
    const c = getColor(colorId);
    document.documentElement.style.setProperty('--note-fill', c.fill);
    document.documentElement.style.setProperty('--note-border', c.border);
    document.documentElement.style.setProperty('--note-text', c.text);
  });

  onMount(async () => {
    const all = await tauri.listNotes();
    const found = all.find((n) => n.id === noteId);
    if (!found) {
      document.body.textContent = 'Note not found';
      return;
    }
    _config = found;
    content = found.content;
    colorId = found.color_id;
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
      }),
    );
  });

  onDestroy(() => {
    cleanup.forEach((fn) => fn());
    if (saveTimer) clearTimeout(saveTimer);
  });

  function onContentInput() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      tauri.updateNoteContent(noteId, content).catch(console.error);
    }, 400);
  }

  async function changeColor(id: ColorId) {
    colorId = id;
    await tauri.updateNoteColor(noteId, id);
  }

  async function changeOpacity(v: number) {
    opacity = v;
    await tauri.updateNoteOpacity(noteId, v);
  }

  async function togglePin() {
    pinned = await tauri.toggleAlwaysOnTop(noteId);
  }

  async function closeNote() {
    if (content.trim().length > 0) {
      const ok = await confirm(
        'This note has content. Close the window? (The note is preserved and stays in the controller list.)',
        { title: 'Close note', kind: 'warning' },
      );
      if (!ok) return;
    }
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
    onTogglePin={togglePin}
    onOpenTimer={() => {
      const el = document.querySelector<HTMLButtonElement>('.timer-bar button');
      el?.click();
    }}
    onClose={closeNote}
  />

  <textarea
    class="content"
    bind:value={content}
    oninput={onContentInput}
    placeholder="Start typing…"
    aria-label="Note content"
  ></textarea>

  <div class="bottom-bar">
    <div class="timer-wrap">
      <Timer {noteId} {timer} {flashing} />
    </div>
    <div class="micro-controls">
      <ColorPicker selected={colorId} onSelect={changeColor} />
      <OpacitySlider value={opacity} onChange={changeOpacity} />
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
    background-color: color-mix(
      in srgb,
      var(--note-fill) calc(var(--note-opacity, 1) * 100%),
      transparent
    );
    border: 0.5px solid var(--note-border);
    border-radius: 8px;
    color: var(--note-text);
    overflow: hidden;
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
