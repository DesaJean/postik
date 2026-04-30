import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { tauri } from '../utils/tauri';
import type { SoundChoice } from '../utils/sound';

export const SETTING_KEYS = {
  privacyHideFromCapture: 'privacy_hide_from_capture',
  soundOnTimerEnd: 'sound_on_timer_end',
  soundChoice: 'sound_choice',
  lastTimerPreset: 'last_timer_preset',
} as const;

const DEFAULTS = {
  privacyHideFromCapture: true,
  soundOnTimerEnd: true,
  soundChoice: 'bell' as SoundChoice,
  lastTimerPreset: '' as string,
} as const;

interface ChangedPayload {
  key: string;
  value: string;
}

const VALID_SOUND_CHOICES: SoundChoice[] = ['bell', 'soft', 'bright'];

class SettingsStore {
  privacyHideFromCapture = $state<boolean>(DEFAULTS.privacyHideFromCapture);
  soundOnTimerEnd = $state<boolean>(DEFAULTS.soundOnTimerEnd);
  soundChoice = $state<SoundChoice>(DEFAULTS.soundChoice);
  lastTimerPreset = $state<string>(DEFAULTS.lastTimerPreset);
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
      this.soundChoice = parseSoundChoice(map.get(SETTING_KEYS.soundChoice));
      this.lastTimerPreset = map.get(SETTING_KEYS.lastTimerPreset) ?? DEFAULTS.lastTimerPreset;
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

  async setSoundChoice(value: SoundChoice) {
    this.soundChoice = value;
    await tauri.setSetting(SETTING_KEYS.soundChoice, value);
  }

  async setLastTimerPreset(value: string) {
    this.lastTimerPreset = value;
    await tauri.setSetting(SETTING_KEYS.lastTimerPreset, value);
  }

  private applyRemote(key: string, value: string) {
    if (key === SETTING_KEYS.privacyHideFromCapture) {
      this.privacyHideFromCapture = value === 'true';
    } else if (key === SETTING_KEYS.soundOnTimerEnd) {
      this.soundOnTimerEnd = value === 'true';
    } else if (key === SETTING_KEYS.soundChoice) {
      this.soundChoice = parseSoundChoice(value);
    } else if (key === SETTING_KEYS.lastTimerPreset) {
      this.lastTimerPreset = value;
    }
  }
}

function parseSoundChoice(raw: string | undefined): SoundChoice {
  if (raw && (VALID_SOUND_CHOICES as string[]).includes(raw)) return raw as SoundChoice;
  return DEFAULTS.soundChoice;
}

function bool(raw: string | undefined, fallback: boolean): boolean {
  if (raw === undefined) return fallback;
  return raw === 'true';
}

export const settingsStore = new SettingsStore();
