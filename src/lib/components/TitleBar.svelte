<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { getColor } from '../utils/colors';
  import type { ColorId } from '../types';

  interface Props {
    colorId: ColorId;
    pinned: boolean;
    onTogglePin: () => void;
    onOpenTimer: () => void;
    onClose: () => void;
  }

  let { colorId, pinned, onTogglePin, onOpenTimer, onClose }: Props = $props();
  let color = $derived(getColor(colorId));

  // `data-tauri-drag-region` is the documented attribute, but it's unreliable
  // on macOS dev builds with transparent windows — WKWebView often doesn't
  // forward the events Tauri's drag listener needs. The programmatic
  // `startDragging()` API works in every config; we trigger it on left-click
  // pointerdown over the drag-handle area.
  function startDrag(event: PointerEvent) {
    if (event.button !== 0) return;
    void getCurrentWindow().startDragging();
  }

  function minimize() {
    void getCurrentWindow().minimize();
  }

  function toggleMaximize() {
    void getCurrentWindow().toggleMaximize();
  }
</script>

<div
  class="title-bar"
  style="--c-border: {color.border};"
  onpointerdown={startDrag}
  role="presentation"
>
  <div class="dots" aria-hidden="true">
    <span class="dot"></span>
    <span class="dot"></span>
  </div>

  <div class="drag-spacer"></div>

  <div class="actions">
    <button
      class="icon-btn"
      class:active={pinned}
      onpointerdown={(e) => e.stopPropagation()}
      onclick={onTogglePin}
      aria-label={pinned ? 'Unpin (always-on-top)' : 'Pin (always-on-top)'}
      title={pinned ? 'Unpin' : 'Pin'}>📌</button
    >
    <button
      class="icon-btn"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={onOpenTimer}
      aria-label="Open timer"
      title="Set timer">⏱</button
    >
    <span class="separator" aria-hidden="true"></span>
    <button
      class="icon-btn"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={minimize}
      aria-label="Minimize"
      title="Minimize"
    >
      <svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true">
        <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1.4" />
      </svg>
    </button>
    <button
      class="icon-btn"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={toggleMaximize}
      aria-label="Maximize"
      title="Maximize"
    >
      <svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true">
        <rect
          x="1.2"
          y="1.2"
          width="7.6"
          height="7.6"
          rx="1"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
        />
      </svg>
    </button>
    <button
      class="icon-btn close"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={onClose}
      aria-label="Close note"
      title="Close">×</button
    >
  </div>
</div>

<style>
  .title-bar {
    display: flex;
    align-items: center;
    height: 28px;
    min-height: 28px;
    padding: 0 8px;
    user-select: none;
    -webkit-user-select: none;
    flex-shrink: 0;
    cursor: grab;
  }
  .title-bar:active {
    cursor: grabbing;
  }
  .dots {
    display: flex;
    gap: 4px;
  }
  .drag-spacer {
    flex: 1;
    height: 100%;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--c-border);
    opacity: 0.5;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .separator {
    width: 1px;
    height: 14px;
    margin: 0 4px;
    background: currentColor;
    opacity: 0.15;
  }
  .icon-btn {
    width: 22px;
    height: 22px;
    border-radius: 3px;
    font-size: 11px;
    opacity: 0.55;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  .icon-btn svg {
    display: block;
  }
  .icon-btn:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.05);
  }
  .icon-btn.active {
    opacity: 1;
  }
  .icon-btn.close {
    font-size: 16px;
  }
  .icon-btn.close:hover {
    background: rgba(216, 90, 48, 0.2);
    color: var(--accent);
  }
</style>
