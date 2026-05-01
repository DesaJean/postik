import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { tauri } from '../utils/tauri';
import type { SoundChoice } from '../utils/sound';

export const SETTING_KEYS = {
  privacyHideFromCapture: 'privacy_hide_from_capture',
  soundOnTimerEnd: 'sound_on_timer_end',
  soundChoice: 'sound_choice',
  lastTimerPreset: 'last_timer_preset',
  lastActionPath: 'last_action_path',
  lastActionArgs: 'last_action_args',
  lastPomodoroCycles: 'last_pomodoro_cycles',
  googleCalendarAutoSync: 'google_calendar_auto_sync',
} as const;

const DEFAULTS = {
  privacyHideFromCapture: true,
  soundOnTimerEnd: true,
  soundChoice: 'bell' as SoundChoice,
  lastTimerPreset: '' as string,
  lastActionPath: '' as string,
  lastActionArgs: '' as string,
  lastPomodoroCycles: 4 as number,
  googleCalendarAutoSync: false,
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
  lastActionPath = $state<string>(DEFAULTS.lastActionPath);
  lastActionArgs = $state<string>(DEFAULTS.lastActionArgs);
  lastPomodoroCycles = $state<number>(DEFAULTS.lastPomodoroCycles);
  googleCalendarAutoSync = $state<boolean>(DEFAULTS.googleCalendarAutoSync);
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
      this.lastActionPath = map.get(SETTING_KEYS.lastActionPath) ?? DEFAULTS.lastActionPath;
      this.lastActionArgs = map.get(SETTING_KEYS.lastActionArgs) ?? DEFAULTS.lastActionArgs;
      this.lastPomodoroCycles = parseCycles(map.get(SETTING_KEYS.lastPomodoroCycles));
      this.googleCalendarAutoSync = bool(
        map.get(SETTING_KEYS.googleCalendarAutoSync),
        DEFAULTS.googleCalendarAutoSync,
      );
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

  async setLastActionPath(value: string) {
    this.lastActionPath = value;
    await tauri.setSetting(SETTING_KEYS.lastActionPath, value);
  }

  async setLastActionArgs(value: string) {
    this.lastActionArgs = value;
    await tauri.setSetting(SETTING_KEYS.lastActionArgs, value);
  }

  async setLastPomodoroCycles(value: number) {
    this.lastPomodoroCycles = value;
    await tauri.setSetting(SETTING_KEYS.lastPomodoroCycles, String(value));
  }

  async setGoogleCalendarAutoSync(value: boolean) {
    this.googleCalendarAutoSync = value;
    await tauri.setSetting(SETTING_KEYS.googleCalendarAutoSync, String(value));
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
    } else if (key === SETTING_KEYS.lastActionPath) {
      this.lastActionPath = value;
    } else if (key === SETTING_KEYS.lastActionArgs) {
      this.lastActionArgs = value;
    } else if (key === SETTING_KEYS.lastPomodoroCycles) {
      this.lastPomodoroCycles = parseCycles(value);
    } else if (key === SETTING_KEYS.googleCalendarAutoSync) {
      this.googleCalendarAutoSync = value === 'true';
    }
  }
}

function parseCycles(raw: string | undefined): number {
  if (!raw) return DEFAULTS.lastPomodoroCycles;
  const n = Number(raw);
  if (!Number.isFinite(n) || n < 1) return DEFAULTS.lastPomodoroCycles;
  return Math.min(8, Math.max(1, Math.floor(n)));
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
