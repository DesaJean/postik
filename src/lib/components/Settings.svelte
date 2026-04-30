<script lang="ts">
  import Switch from './Switch.svelte';
  import { settingsStore } from '../stores/settings.svelte';

  interface Props {
    onBack: () => void;
  }

  let { onBack }: Props = $props();
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
            Plays a soft chime that loops until you click Dismiss on the finished timer.
          </div>
        </div>
        <Switch
          checked={settingsStore.soundOnTimerEnd}
          onChange={(v) => settingsStore.setSoundOnTimerEnd(v)}
          label="Sound on timer end"
        />
      </div>
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
  }
</style>
