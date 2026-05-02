<script lang="ts">
  import { COLORS, TEXT_COLORS, getColor } from '../utils/colors';
  import type { ColorId, TextColorId } from '../types';

  interface Props {
    colorId: ColorId;
    opacity: number;
    textColorId: TextColorId;
    onColorChange: (id: ColorId) => void;
    onOpacityChange: (value: number) => void;
    onTextColorChange: (id: TextColorId) => void;
  }

  let { colorId, opacity, textColorId, onColorChange, onOpacityChange, onTextColorChange }: Props =
    $props();

  let paletteText = $derived(getColor(colorId).text);

  let open = $state(false);
  let popoverEl: HTMLDivElement | undefined = $state();

  function toggle() {
    open = !open;
  }

  function close() {
    open = false;
  }

  function onWindowMouseDown(e: MouseEvent) {
    if (!open) return;
    const target = e.target as Node;
    if (popoverEl && !popoverEl.contains(target)) {
      close();
    }
  }

  function onOpacityInput(e: Event) {
    const v = parseFloat((e.target as HTMLInputElement).value);
    onOpacityChange(v);
  }
</script>

<svelte:window onmousedown={onWindowMouseDown} onkeydown={(e) => e.key === 'Escape' && close()} />

<div class="appearance">
  <button
    class="trigger"
    class:active={open}
    onclick={toggle}
    aria-label="Appearance"
    aria-expanded={open}
    title="Appearance"
  >
    <svg viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
      <path
        d="M8 1.5C4.41 1.5 1.5 4.41 1.5 8c0 1.66.61 3.17 1.62 4.32.31.36.78.45 1.18.21.41-.24.62-.7.55-1.16-.05-.32-.07-.59-.07-.87 0-.55.45-1 1-1h.5c.83 0 1.5-.67 1.5-1.5S6.83 6.5 6 6.5 4.5 5.83 4.5 5 5.17 3.5 6 3.5h2c2.49 0 4.5 2.01 4.5 4.5 0 1.93-1.21 3.59-2.92 4.23-.27.1-.45.36-.45.65v.62c0 .55.45 1 1 1 3.59 0 6.5-2.91 6.5-6.5S11.59 1.5 8 1.5z"
        fill="currentColor"
        opacity="0.85"
      />
    </svg>
  </button>

  {#if open}
    <div class="popover" bind:this={popoverEl} role="dialog" aria-label="Appearance">
      <div class="section">
        <div class="section-heading">Color</div>
        <div class="swatches">
          {#each COLORS as c (c.id)}
            <button
              class="swatch"
              class:selected={c.id === colorId}
              class:transparent-swatch={c.id === 'transparent'}
              style="background: {c.fill === 'transparent'
                ? 'transparent'
                : c.fill}; --c-border: {c.border};"
              onclick={() => onColorChange(c.id)}
              aria-label={c.id}
              aria-pressed={c.id === colorId}
              title={c.id}
            ></button>
          {/each}
        </div>
      </div>

      <div class="divider" aria-hidden="true"></div>

      <div class="section">
        <div class="section-heading">Text</div>
        <div class="text-swatches">
          {#each TEXT_COLORS as t (t.id)}
            <button
              class="text-swatch"
              class:selected={t.id === textColorId}
              class:auto={t.id === 'auto'}
              style="--swatch-color: {t.value ?? paletteText};"
              onclick={() => onTextColorChange(t.id)}
              aria-label={`Text color: ${t.label}`}
              aria-pressed={t.id === textColorId}
              title={t.label}
            >
              <span class="text-swatch-glyph">A</span>
            </button>
          {/each}
        </div>
      </div>

      <div class="divider" aria-hidden="true"></div>

      <div class="section">
        <div class="section-heading-row">
          <span class="section-heading">Opacity</span>
          <span class="opacity-value">{Math.round(opacity * 100)}%</span>
        </div>
        <div class="opacity-presets">
          <button
            class="opacity-chip"
            class:active={Math.abs(opacity - 0.4) < 0.025}
            onclick={() => onOpacityChange(0.4)}
            title="Ghost (40%)"
          >
            Ghost
          </button>
          <button
            class="opacity-chip"
            class:active={Math.abs(opacity - 0.7) < 0.025}
            onclick={() => onOpacityChange(0.7)}
            title="Normal (70%)"
          >
            Normal
          </button>
          <button
            class="opacity-chip"
            class:active={Math.abs(opacity - 1.0) < 0.025}
            onclick={() => onOpacityChange(1)}
            title="Opaque (100%)"
          >
            Opaque
          </button>
        </div>
        <input
          class="opacity-slider"
          type="range"
          min="0.2"
          max="1"
          step="0.05"
          value={opacity}
          oninput={onOpacityInput}
          aria-label="Opacity"
        />
        <p class="opacity-hint">Hover a faded note to temporarily see it at 100%.</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .appearance {
    position: relative;
  }

  .trigger {
    width: 22px;
    height: 22px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: currentColor;
    opacity: 0.55;
    transition:
      opacity 120ms ease-out,
      background-color 120ms ease-out;
  }
  .trigger:hover,
  .trigger.active {
    opacity: 1;
    background: rgba(0, 0, 0, 0.06);
  }

  .popover {
    position: absolute;
    bottom: calc(100% + 6px);
    right: 0;
    min-width: 196px;
    background: rgba(255, 255, 255, 0.98);
    border: 1px solid rgba(0, 0, 0, 0.08);
    border-radius: 8px;
    box-shadow:
      0 1px 2px rgba(0, 0, 0, 0.04),
      0 8px 24px rgba(0, 0, 0, 0.12);
    padding: 10px 12px 12px;
    z-index: 20;
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    animation: pop-in 140ms cubic-bezier(0.4, 0, 0.2, 1);
  }
  @keyframes pop-in {
    from {
      opacity: 0;
      transform: translateY(2px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-heading {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgba(0, 0, 0, 0.45);
  }
  .section-heading-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .opacity-value {
    font-size: 11px;
    font-weight: 500;
    color: rgba(0, 0, 0, 0.6);
    font-variant-numeric: tabular-nums;
  }

  .divider {
    height: 1px;
    background: rgba(0, 0, 0, 0.08);
    margin: 10px 0;
  }

  .swatches {
    display: grid;
    grid-template-columns: repeat(7, 20px);
    gap: 6px;
  }

  .text-swatches {
    display: grid;
    grid-template-columns: repeat(5, 26px);
    gap: 6px;
  }
  .text-swatch {
    width: 26px;
    height: 26px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.04);
    border: 1px solid rgba(0, 0, 0, 0.08);
    color: var(--swatch-color);
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out;
  }
  .text-swatch-glyph {
    font-size: 13px;
    font-weight: 700;
    line-height: 1;
  }
  .text-swatch:hover {
    background: rgba(0, 0, 0, 0.08);
  }
  .text-swatch.selected {
    border-color: var(--accent);
    background: rgba(216, 90, 48, 0.08);
  }
  .text-swatch.auto .text-swatch-glyph {
    background: linear-gradient(45deg, var(--swatch-color) 50%, var(--accent) 50%);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .swatch {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 1px solid var(--c-border);
    cursor: pointer;
    padding: 0;
    transition:
      transform 120ms cubic-bezier(0.4, 0, 0.2, 1),
      box-shadow 120ms ease-out;
  }
  .swatch:hover {
    transform: scale(1.12);
  }
  .swatch.selected {
    box-shadow:
      0 0 0 2px white,
      0 0 0 3.5px var(--c-border);
  }
  .swatch.transparent-swatch {
    background-image:
      linear-gradient(
        45deg,
        rgba(0, 0, 0, 0.08) 25%,
        transparent 25%,
        transparent 75%,
        rgba(0, 0, 0, 0.08) 75%
      ),
      linear-gradient(
        45deg,
        rgba(0, 0, 0, 0.08) 25%,
        transparent 25%,
        transparent 75%,
        rgba(0, 0, 0, 0.08) 75%
      ) !important;
    background-size: 6px 6px;
    background-position:
      0 0,
      3px 3px;
  }

  .opacity-presets {
    display: flex;
    gap: 4px;
    margin-bottom: 6px;
  }
  .opacity-chip {
    flex: 1;
    height: 22px;
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
  .opacity-chip:hover {
    background: rgba(216, 90, 48, 0.08);
    color: var(--accent);
  }
  .opacity-chip.active {
    background: rgba(216, 90, 48, 0.18);
    color: var(--accent);
  }
  .opacity-hint {
    margin: 6px 0 0;
    font-size: 9.5px;
    color: var(--text-muted);
    line-height: 1.3;
  }

  .opacity-slider {
    width: 100%;
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    background: rgba(0, 0, 0, 0.1);
    border-radius: 2px;
    outline: none;
    margin: 0;
  }
  .opacity-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: white;
    border: 1px solid rgba(0, 0, 0, 0.15);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18);
    cursor: pointer;
    transition: transform 100ms ease-out;
  }
  .opacity-slider::-webkit-slider-thumb:hover {
    transform: scale(1.1);
  }
  .opacity-slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: white;
    border: 1px solid rgba(0, 0, 0, 0.15);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18);
    cursor: pointer;
  }

  @media (prefers-color-scheme: dark) {
    .popover {
      background: rgba(40, 40, 38, 0.95);
      border-color: rgba(255, 255, 255, 0.1);
      color: #f1f0ec;
    }
    .section-heading {
      color: rgba(255, 255, 255, 0.5);
    }
    .opacity-value {
      color: rgba(255, 255, 255, 0.7);
    }
    .divider {
      background: rgba(255, 255, 255, 0.1);
    }
    .opacity-slider {
      background: rgba(255, 255, 255, 0.15);
    }
    .trigger:hover,
    .trigger.active {
      background: rgba(255, 255, 255, 0.1);
    }
    .text-swatch {
      background: rgba(255, 255, 255, 0.06);
      border-color: rgba(255, 255, 255, 0.1);
    }
    .text-swatch:hover {
      background: rgba(255, 255, 255, 0.12);
    }
    .text-swatch.selected {
      background: rgba(216, 90, 48, 0.18);
    }
  }
</style>
