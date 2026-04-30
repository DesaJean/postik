<script lang="ts">
  interface Props {
    checked: boolean;
    onChange: (next: boolean) => void;
    label: string;
    disabled?: boolean;
  }

  let { checked, onChange, label, disabled = false }: Props = $props();

  function toggle() {
    if (disabled) return;
    onChange(!checked);
  }

  function onKey(e: KeyboardEvent) {
    if (disabled) return;
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault();
      toggle();
    }
  }
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  class="switch"
  class:on={checked}
  class:disabled
  onclick={toggle}
  onkeydown={onKey}
>
  <span class="thumb" aria-hidden="true"></span>
</button>

<style>
  .switch {
    position: relative;
    width: 32px;
    height: 18px;
    border-radius: 9px;
    background: rgba(0, 0, 0, 0.18);
    padding: 0;
    border: none;
    cursor: pointer;
    flex-shrink: 0;
    transition: background-color 200ms ease-out;
  }
  .switch:hover:not(.disabled) {
    background: rgba(0, 0, 0, 0.28);
  }
  .switch.on {
    background: var(--accent);
  }
  .switch.on:hover:not(.disabled) {
    background: #c64f29;
  }
  .switch.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .switch:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.18);
    transition: transform 150ms cubic-bezier(0.4, 0, 0.2, 1);
  }
  .switch.on .thumb {
    transform: translateX(14px);
  }

  @media (prefers-color-scheme: dark) {
    .switch {
      background: rgba(255, 255, 255, 0.2);
    }
    .switch:hover:not(.disabled) {
      background: rgba(255, 255, 255, 0.3);
    }
  }
</style>
