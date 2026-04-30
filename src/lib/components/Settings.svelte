<script lang="ts">
  import Switch from './Switch.svelte';
  import { settingsStore } from '../stores/settings.svelte';
  import { SOUND_CHOICES, playTimerDone, type SoundChoice } from '../utils/sound';

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();

  function selectSound(id: SoundChoice) {
    settingsStore.setSoundChoice(id);
    // Preview the chosen variant immediately so the user hears what they
    // picked without setting up a real timer.
    playTimerDone(id);
  }
</script>

<div class="settings-view">
  <header class="header">
    <button class="back-btn" onclick={onBack} aria-label="Back to notes" title="Back">
      <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
        <path
          d="M10 3L5 8L10 13"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
    <h1>Settings</h1>
  </header>

  <div class="content">
    <section>
      <h2 class="section-heading">Privacy</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Hide from screen sharing</div>
          <div class="row-helper">
            Postik windows render as black rectangles in screen captures (Zoom, Meet, Teams,
            QuickTime). The tray icon stays visible — use ⌘⇧H to hide it too.
          </div>
        </div>
        <Switch
          checked={settingsStore.privacyHideFromCapture}
          onChange={(v) => settingsStore.setPrivacyHideFromCapture(v)}
          label="Hide from screen sharing"
        />
      </div>
    </section>

    <section>
      <h2 class="section-heading">Playback</h2>
      <div class="row">
        <div class="row-text">
          <div class="row-label">Sound on timer end</div>
          <div class="row-helper">
            Plays a chime that loops until you click Dismiss on the finished timer.
          </div>
        </div>
        <Switch
          checked={settingsStore.soundOnTimerEnd}
          onChange={(v) => settingsStore.setSoundOnTimerEnd(v)}
          label="Sound on timer end"
        />
      </div>

      {#if settingsStore.soundOnTimerEnd}
        <div class="sound-row">
          <div class="row-label sound-row-label">Chime</div>
          <div class="sound-list">
            {#each SOUND_CHOICES as choice (choice.id)}
              <button
                class="sound-option"
                class:selected={settingsStore.soundChoice === choice.id}
                onclick={() => selectSound(choice.id)}
                aria-pressed={settingsStore.soundChoice === choice.id}
              >
                <div class="sound-radio" aria-hidden="true">
                  {#if settingsStore.soundChoice === choice.id}
                    <div class="sound-radio-dot"></div>
                  {/if}
                </div>
                <div class="sound-text">
                  <div class="sound-label">{choice.label}</div>
                  <div class="sound-desc">{choice.description}</div>
                </div>
                <span class="sound-preview-hint" aria-hidden="true">▸</span>
              </button>
            {/each}
          </div>
          <div class="row-helper sound-helper">Click any option to preview and select.</div>
        </div>
      {/if}
    </section>

    <p class="footer-note">
      More settings — sound choice, default color, custom shortcuts — in v0.2.
    </p>
  </div>
</div>

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .header {
    display: flex;
    align-items: center;
    height: 44px;
    padding: 0 12px;
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
    gap: 8px;
  }
  .header h1 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
    letter-spacing: -0.01em;
  }
  .back-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    color: var(--text-muted);
    transition: background-color 120ms ease-out;
  }
  .back-btn:hover {
    background: rgba(0, 0, 0, 0.05);
    color: inherit;
  }

  .content {
    flex: 1;
    overflow-y: auto;
  }

  section {
    padding-bottom: 8px;
  }
  .section-heading {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    margin: 0;
    padding: 16px 16px 8px;
  }

  .row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 10px 16px;
  }
  .row-text {
    flex: 1;
    min-width: 0;
  }
  .row-label {
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 4px;
  }
  .row-helper {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .sound-row {
    padding: 0 16px 12px;
  }
  .sound-row-label {
    margin-bottom: 6px;
  }
  .sound-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .sound-option {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid transparent;
    cursor: pointer;
    width: 100%;
    text-align: left;
    transition:
      background-color 120ms ease-out,
      border-color 120ms ease-out;
  }
  .sound-option:hover {
    background: rgba(216, 90, 48, 0.05);
  }
  .sound-option.selected {
    background: rgba(216, 90, 48, 0.08);
    border-color: rgba(216, 90, 48, 0.3);
  }
  .sound-radio {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 1.5px solid rgba(0, 0, 0, 0.25);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color 120ms ease-out;
  }
  .sound-option.selected .sound-radio {
    border-color: var(--accent);
  }
  .sound-radio-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
  }
  .sound-text {
    flex: 1;
    min-width: 0;
  }
  .sound-label {
    font-size: 12px;
    font-weight: 500;
  }
  .sound-desc {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: 1px;
  }
  .sound-preview-hint {
    font-size: 10px;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity 120ms ease-out;
  }
  .sound-option:hover .sound-preview-hint {
    opacity: 0.7;
  }
  .sound-helper {
    margin-top: 8px;
  }
  .footer-note {
    font-size: 11px;
    color: var(--text-muted);
    text-align: center;
    margin: 24px 16px 16px;
  }

  @media (prefers-color-scheme: dark) {
    .back-btn:hover {
      background: rgba(255, 255, 255, 0.08);
    }
    .sound-option {
      background: rgba(255, 255, 255, 0.04);
    }
    .sound-option:hover {
      background: rgba(216, 90, 48, 0.18);
    }
    .sound-option.selected {
      background: rgba(216, 90, 48, 0.22);
    }
    .sound-radio {
      border-color: rgba(255, 255, 255, 0.3);
    }
  }
</style>
