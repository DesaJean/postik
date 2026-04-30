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

<div class="title-bar" data-tauri-drag-region style="--c-border: {color.border};">
  <div class="dots" aria-hidden="true">
    <span class="dot"></span>
    <span class="dot"></span>
  </div>

  <div class="drag-spacer" data-tauri-drag-region></div>

  <div class="actions">
    <button
      class="icon-btn"
      class:active={pinned}
      onclick={onTogglePin}
      aria-label={pinned ? 'Unpin (always-on-top)' : 'Pin (always-on-top)'}
      title={pinned ? 'Unpin' : 'Pin'}>📌</button
    >
    <button class="icon-btn" onclick={onOpenTimer} aria-label="Open timer" title="Set timer"
      >⏱</button
    >
    <button class="icon-btn close" onclick={onClose} aria-label="Close note" title="Close">×</button
    >
  </div>
</div>

<style>
  .title-bar {
    display: flex;
    align-items: center;
    height: 28px;
    padding: 0 8px;
    user-select: none;
    -webkit-user-select: none;
    flex-shrink: 0;
  }
  .dots {
    display: flex;
    gap: 4px;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--c-border);
    opacity: 0.5;
  }
  .drag-spacer {
    flex: 1;
    height: 100%;
  }
  .actions {
    display: flex;
    gap: 2px;
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
