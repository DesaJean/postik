import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { tauri } from '../utils/tauri';

export const SETTING_KEYS = {
  privacyHideFromCapture: 'privacy_hide_from_capture',
  soundOnTimerEnd: 'sound_on_timer_end',
} as const;

const DEFAULTS = {
  privacyHideFromCapture: true,
  soundOnTimerEnd: true,
} as const;

interface ChangedPayload {
  key: string;
  value: string;
}

class SettingsStore {
  privacyHideFromCapture = $state<boolean>(DEFAULTS.privacyHideFromCapture);
  soundOnTimerEnd = $state<boolean>(DEFAULTS.soundOnTimerEnd);
  loaded = $state(false);

  private unlisten: UnlistenFn | null = null;

  async load() {
    try {
      const all = await tauri.listSettings();
      const map = new Map(all.map((p) => [p.key, p.value]));
      this.privacyHideFromCapture = bool(
        map.get(SETTING_KEYS.privacyHideFromCapture),
        DEFAULTS.privacyHideFromCapture,
      );
      this.soundOnTimerEnd = bool(map.get(SETTING_KEYS.soundOnTimerEnd), DEFAULTS.soundOnTimerEnd);
      this.loaded = true;
    } catch (e) {
      console.error('Failed to load settings:', e);
    }

    if (!this.unlisten) {
      this.unlisten = await listen<ChangedPayload>('settings:changed', (event) => {
        this.applyRemote(event.payload.key, event.payload.value);
      });
    }
  }

  async setPrivacyHideFromCapture(value: boolean) {
    this.privacyHideFromCapture = value;
    await tauri.setSetting(SETTING_KEYS.privacyHideFromCapture, String(value));
  }

  async setSoundOnTimerEnd(value: boolean) {
    this.soundOnTimerEnd = value;
    await tauri.setSetting(SETTING_KEYS.soundOnTimerEnd, String(value));
  }

  private applyRemote(key: string, value: string) {
    if (key === SETTING_KEYS.privacyHideFromCapture) {
      this.privacyHideFromCapture = value === 'true';
    } else if (key === SETTING_KEYS.soundOnTimerEnd) {
      this.soundOnTimerEnd = value === 'true';
    }
  }
}

function bool(raw: string | undefined, fallback: boolean): boolean {
  if (raw === undefined) return fallback;
  return raw === 'true';
}

export const settingsStore = new SettingsStore();
