<script lang="ts">
  import { COLORS } from '../utils/colors';
  import type { ColorId } from '../types';

  interface Props {
    selected: ColorId;
    onSelect: (id: ColorId) => void;
  }

  let { selected, onSelect }: Props = $props();
  let open = $state(false);
</script>

<div class="picker">
  <button class="trigger" onclick={() => (open = !open)} aria-label="Change color" title="Color"
    >🎨</button
  >

  {#if open}
    <div class="popover" role="menu">
      {#each COLORS as c (c.id)}
        <button
          class="swatch"
          class:selected={c.id === selected}
          style="background: {c.fill}; border-color: {c.border};"
          onclick={() => {
            onSelect(c.id);
            open = false;
          }}
          aria-label={c.id}
          title={c.id}
        ></button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
  }
  .trigger {
    width: 18px;
    height: 18px;
    border-radius: 3px;
    font-size: 11px;
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
    padding: 6px;
    display: grid;
    grid-template-columns: repeat(4, 18px);
    gap: 4px;
    z-index: 10;
  }
  .swatch {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 1px solid;
  }
  .swatch.selected {
    box-shadow: 0 0 0 2px var(--accent);
  }
</style>
