<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { getColor } from '../utils/colors';
  import type { ColorId } from '../types';

  interface Props {
    colorId: ColorId;
    pinned: boolean;
    focusMode: boolean;
    onTogglePin: () => void;
    onToggleFocus: () => void;
    onOpenTimer: () => void;
    onClose: () => void;
  }

  let { colorId, pinned, focusMode, onTogglePin, onToggleFocus, onOpenTimer, onClose }: Props =
    $props();
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

  <!-- Postik logo, simplified for 14px size: ring + sticky note + P -->
  <div class="logo-mark" aria-hidden="true">
    <svg viewBox="0 0 16 16" width="14" height="14">
      <circle
        cx="8"
        cy="8"
        r="6.5"
        fill="none"
        stroke="currentColor"
        stroke-width="0.9"
        opacity="0.25"
      />
      <path
        d="M 8 1.5 A 6.5 6.5 0 0 1 14.1 9.5"
        fill="none"
        stroke="#D85A30"
        stroke-width="1.2"
        stroke-linecap="round"
      />
      <rect
        x="4.5"
        y="4.5"
        width="7"
        height="7"
        rx="1"
        fill="#FAC775"
        stroke="#854F0B"
        stroke-width="0.4"
      />
      <text
        x="8"
        y="9.7"
        text-anchor="middle"
        fill="#412402"
        style="font-size: 5.5px; font-weight: 700; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;"
        >P</text
      >
    </svg>
  </div>

  <div class="drag-spacer"></div>

  <div class="actions">
    <button
      class="icon-btn"
      class:active={pinned}
      onpointerdown={(e) => e.stopPropagation()}
      onclick={onTogglePin}
      aria-label={pinned ? 'Unpin (always-on-top)' : 'Pin (always-on-top)'}
      title={pinned ? 'Unpin' : 'Pin'}
    >
      <!-- Pin / thumbtack icon, lucide-style -->
      <svg viewBox="0 0 16 16" width="11" height="11" aria-hidden="true">
        <path
          d="M8 11.5v3M5 11.5h6m-1-7v3.6c0 .5.25.97.66 1.25l1.18.8c.42.28.66.75.66 1.26V11.5H3.5v-1.09c0-.5.25-.98.66-1.26l1.18-.8a1.5 1.5 0 0 0 .66-1.25V4.5h-.5a1 1 0 1 1 0-2h5a1 1 0 1 1 0 2H10z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <button
      class="icon-btn"
      class:active={focusMode}
      onpointerdown={(e) => e.stopPropagation()}
      onclick={onToggleFocus}
      aria-label={focusMode ? 'Exit focus mode' : 'Focus this note (hide others)'}
      title={focusMode ? 'Exit focus mode' : 'Focus mode'}
    >
      <!-- Eye icon: open when focus is off, slashed when on (you're hiding others) -->
      <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
        {#if focusMode}
          <path
            d="M3 3l10 10M2.5 8s2-4 5.5-4c1 0 1.9.3 2.6.7M13.5 8s-1 2-3 3.2M7 5.4a3 3 0 0 1 3.6 3.6"
            fill="none"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linecap="round"
          />
        {:else}
          <path
            d="M2.5 8s2-4 5.5-4 5.5 4 5.5 4-2 4-5.5 4-5.5-4-5.5-4z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.2"
          />
          <circle cx="8" cy="8" r="1.6" fill="currentColor" />
        {/if}
      </svg>
    </button>
    <button
      class="icon-btn"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={onOpenTimer}
      aria-label="Open timer"
      title="Set timer"
    >
      <!-- Clock icon -->
      <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
        <circle cx="8" cy="8.5" r="5.5" fill="none" stroke="currentColor" stroke-width="1.2" />
        <path
          d="M8 5.5V8.5L9.8 9.6"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <path
          d="M6.5 2L8 1L9.5 2"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
        />
      </svg>
    </button>
    <span class="separator" aria-hidden="true"></span>
    <button
      class="icon-btn"
      onpointerdown={(e) => e.stopPropagation()}
      onclick={minimize}
      aria-label="Minimize"
      title="Minimize"
    >
      <svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true">
        <line
          x1="1.5"
          y1="5"
          x2="8.5"
          y2="5"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        />
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
          x="1.5"
          y="1.5"
          width="7"
          height="7"
          rx="1.2"
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
      title="Close"
    >
      <svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true">
        <path
          d="M2 2L8 8M8 2L2 8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        />
      </svg>
    </button>
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
  .logo-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.85;
    pointer-events: none;
    flex-shrink: 0;
  }
  .logo-mark svg {
    display: block;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--c-border);
    opacity: 0.4;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 1px;
  }
  .separator {
    width: 1px;
    height: 12px;
    margin: 0 4px;
    background: currentColor;
    opacity: 0.18;
  }
  .icon-btn {
    width: 22px;
    height: 22px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: currentColor;
    opacity: 0.55;
    cursor: pointer;
    transition:
      opacity 120ms ease-out,
      background-color 120ms ease-out;
  }
  .icon-btn svg {
    display: block;
  }
  .icon-btn:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.06);
  }
  .icon-btn.active {
    opacity: 1;
    color: var(--accent);
  }
  .icon-btn.close:hover {
    background: rgba(216, 90, 48, 0.18);
    color: var(--accent);
  }

  @media (prefers-color-scheme: dark) {
    .icon-btn:hover {
      background: rgba(255, 255, 255, 0.08);
    }
  }
</style>
