import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { tauri } from '../utils/tauri';
import type { GoogleAccountInfo, GoogleEventRecord, SyncRangeKind } from '../types';

class OutlookStore {
  isConfigured = $state<boolean>(true);
  account = $state<GoogleAccountInfo | null>(null);
  events = $state<GoogleEventRecord[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  range = $state<SyncRangeKind>('today');

  private unlistenAccount: UnlistenFn | null = null;
  private unlistenEvents: UnlistenFn | null = null;

  async load() {
    try {
      this.isConfigured = await tauri.outlookIsConfigured();
      this.account = await tauri.outlookAccount();
      this.events = await tauri.outlookListEvents();
    } catch (e) {
      this.error = String(e);
    }
    if (!this.unlistenAccount) {
      this.unlistenAccount = await listen<GoogleAccountInfo | null>(
        'outlook:account-changed',
        (e) => {
          this.account = e.payload;
          if (!e.payload) this.events = [];
        },
      );
    }
    if (!this.unlistenEvents) {
      this.unlistenEvents = await listen<GoogleEventRecord[]>('outlook:events-synced', (e) => {
        this.events = e.payload;
      });
    }
  }

  async connect() {
    this.error = null;
    this.loading = true;
    try {
      this.account = await tauri.outlookConnect();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  async disconnect() {
    this.error = null;
    try {
      await tauri.outlookDisconnect();
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
      this.events = await tauri.outlookSync(this.range);
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
}

export const outlookStore = new OutlookStore();
