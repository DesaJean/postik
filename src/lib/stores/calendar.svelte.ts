import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { tauri } from '../utils/tauri';
import type { GoogleAccountInfo, GoogleEventRecord, SyncRangeKind } from '../types';

class CalendarStore {
  isConfigured = $state<boolean>(true); // assume true until first check resolves
  account = $state<GoogleAccountInfo | null>(null);
  events = $state<GoogleEventRecord[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  range = $state<SyncRangeKind>('today');

  private unlistenAccount: UnlistenFn | null = null;
  private unlistenEvents: UnlistenFn | null = null;

  async load() {
    try {
      this.isConfigured = await tauri.googleIsConfigured();
      this.account = await tauri.googleAccount();
      this.events = await tauri.googleListEvents();
    } catch (e) {
      this.error = String(e);
    }

    if (!this.unlistenAccount) {
      this.unlistenAccount = await listen<GoogleAccountInfo | null>(
        'google:account-changed',
        (e) => {
          this.account = e.payload;
          if (!e.payload) {
            this.events = [];
          }
        },
      );
    }
    if (!this.unlistenEvents) {
      this.unlistenEvents = await listen<GoogleEventRecord[]>('google:events-synced', (e) => {
        this.events = e.payload;
      });
    }
  }

  async connect() {
    this.error = null;
    this.loading = true;
    try {
      this.account = await tauri.googleConnect();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async disconnect() {
    this.error = null;
    try {
      await tauri.googleDisconnect();
      this.account = null;
      this.events = [];
    } catch (e) {
      this.error = String(e);
    }
  }

  async sync() {
    if (!this.account) return;
    this.error = null;
    this.loading = true;
    try {
      this.events = await tauri.googleSync(this.range);
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async setRange(range: SyncRangeKind) {
    this.range = range;
    await this.sync();
  }

  async setEventTimer(eventId: string, armed: boolean, offsetSeconds: number) {
    try {
      await tauri.googleSetEventTimer(eventId, armed, offsetSeconds);
      this.events = this.events.map((e) =>
        e.event_id === eventId
          ? { ...e, timer_armed: armed, timer_offset_seconds: offsetSeconds }
          : e,
      );
    } catch (e) {
      this.error = String(e);
    }
  }

  async openEvent(eventId: string) {
    try {
      await tauri.googleOpenEvent(eventId);
    } catch (e) {
      this.error = String(e);
    }
  }
}

export const calendarStore = new CalendarStore();
