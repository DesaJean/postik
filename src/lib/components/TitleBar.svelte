<script lang="ts">
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
</script>

<!--
  Drag handling: Tauri's `data-tauri-drag-region` is the documented attribute,
  but in some dev-mode + transparent-window combinations on macOS it doesn't
  fire reliably; pairing it with `-webkit-app-region: drag` (the legacy
  Electron/Chromium hint, which Tauri's WebView also honors) makes drag
  consistent. Buttons must opt OUT explicitly with both `data-tauri-drag-region="false"`
  AND `-webkit-app-region: no-drag` or clicks get swallowed by the drag area.
-->
<div class="title-bar" data-tauri-drag-region style="--c-border: {color.border};">
  <div class="dots" data-tauri-drag-region aria-hidden="true">
    <span class="dot" data-tauri-drag-region></span>
    <span class="dot" data-tauri-drag-region></span>
  </div>

  <div class="drag-spacer" data-tauri-drag-region></div>

  <div class="actions">
    <button
      class="icon-btn"
      class:active={pinned}
      data-tauri-drag-region="false"
      onclick={onTogglePin}
      aria-label={pinned ? 'Unpin (always-on-top)' : 'Pin (always-on-top)'}
      title={pinned ? 'Unpin' : 'Pin'}>📌</button
    >
    <button
      class="icon-btn"
      data-tauri-drag-region="false"
      onclick={onOpenTimer}
      aria-label="Open timer"
      title="Set timer">⏱</button
    >
    <button
      class="icon-btn close"
      data-tauri-drag-region="false"
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
    -webkit-app-region: drag;
  }
  .title-bar:active {
    cursor: grabbing;
  }
  .dots {
    display: flex;
    gap: 4px;
    -webkit-app-region: drag;
  }
  .drag-spacer {
    flex: 1;
    height: 100%;
    -webkit-app-region: drag;
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
    gap: 2px;
    -webkit-app-region: no-drag;
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
    -webkit-app-region: no-drag;
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
