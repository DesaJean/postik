<script lang="ts">
  interface Props {
    value: number;
    onChange: (v: number) => void;
  }

  let { value, onChange }: Props = $props();
  let open = $state(false);

  function handleInput(e: Event) {
    const v = parseFloat((e.target as HTMLInputElement).value);
    onChange(v);
  }
</script>

<div class="opacity">
  <button class="trigger" onclick={() => (open = !open)} aria-label="Adjust opacity" title="Opacity"
    >◐</button
  >

  {#if open}
    <div class="popover">
      <input
        type="range"
        min="0.2"
        max="1"
        step="0.05"
        {value}
        oninput={handleInput}
        aria-label="Opacity"
      />
      <span>{Math.round(value * 100)}%</span>
    </div>
  {/if}
</div>

<style>
  .opacity {
    position: relative;
  }
  .trigger {
    width: 18px;
    height: 18px;
    border-radius: 3px;
    font-size: 12px;
    opacity: 0.55;
  }
  .trigger:hover {
    opacity: 1;
  }
  .popover {
    position: absolute;
    bottom: 24px;
    right: 0;
    background: white;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 6px;
    padding: 6px 8px;
    display: flex;
    align-items: center;
    gap: 8px;
    z-index: 10;
    width: 140px;
  }
  input[type='range'] {
    flex: 1;
  }
  span {
    font-size: 11px;
    color: rgba(0, 0, 0, 0.6);
    min-width: 30px;
    text-align: right;
  }
</style>
