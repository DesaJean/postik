import { invoke } from '@tauri-apps/api/core';
import type {
  ColorId,
  GoogleAccountInfo,
  GoogleEventRecord,
  NoteConfig,
  StackRecord,
  SyncRangeKind,
  TextColorId,
  TimerMode,
  TimerStatePayload,
} from '../types';

export const tauri = {
  createNote: (initialPosition?: [number, number]) =>
    invoke<NoteConfig>('create_note', { initialPosition: initialPosition ?? null }),

  updateNoteContent: (noteId: string, content: string) =>
    invoke<void>('update_note_content', { noteId, content }),

  updateNoteColor: (noteId: string, colorId: ColorId) =>
    invoke<void>('update_note_color', { noteId, colorId }),

  updateNoteTextColor: (noteId: string, textColor: TextColorId | null) =>
    invoke<void>('update_note_text_color', { noteId, textColor }),

  updateNoteTags: (noteId: string, tags: string | null) =>
    invoke<void>('update_note_tags', { noteId, tags }),

  updateNoteRecurringRule: (noteId: string, rule: string | null) =>
    invoke<void>('update_note_recurring_rule', { noteId, rule }),

  listStacks: () => invoke<StackRecord[]>('list_stacks'),
  createStack: (name: string, color: string | null = null) =>
    invoke<StackRecord>('create_stack', { name, color }),
  updateStack: (id: string, name: string, color: string | null = null) =>
    invoke<void>('update_stack', { id, name, color }),
  deleteStack: (id: string) => invoke<void>('delete_stack', { id }),
  setNoteStack: (noteId: string, stackId: string | null) =>
    invoke<void>('set_note_stack', { noteId, stackId }),

  updateNoteOpacity: (noteId: string, opacity: number) =>
    invoke<void>('update_note_opacity', { noteId, opacity }),

  updateNotePosition: (noteId: string, x: number, y: number) =>
    invoke<void>('update_note_position', { noteId, x, y }),

  updateNoteSize: (noteId: string, width: number, height: number) =>
    invoke<void>('update_note_size', { noteId, width, height }),

  toggleAlwaysOnTop: (noteId: string) => invoke<boolean>('toggle_always_on_top', { noteId }),

  deleteNote: (noteId: string) => invoke<void>('delete_note', { noteId }),
  archiveNote: (noteId: string) => invoke<void>('archive_note', { noteId }),
  unarchiveNote: (noteId: string) => invoke<void>('unarchive_note', { noteId }),
  listArchivedNotes: () => invoke<NoteConfig[]>('list_archived_notes'),

  listNotes: () => invoke<NoteConfig[]>('list_notes'),

  reorderNotes: (orderedIds: string[]) => invoke<void>('reorder_notes', { orderedIds }),

  focusNote: (noteId: string) => invoke<void>('focus_note', { noteId }),

  hideAllNotes: () => invoke<void>('hide_all_notes'),
  showAllNotes: () => invoke<void>('show_all_notes'),
  focusOnlyNote: (noteId: string) => invoke<void>('focus_only_note', { noteId }),

  startTimer: (
    noteId: string,
    mode: TimerMode,
    durationSeconds: number | null,
    options?: {
      pomodoroCycles?: number | null;
      actionPath?: string | null;
      actionArgs?: string | null;
      webhookUrl?: string | null;
    },
  ) =>
    invoke<void>('start_timer', {
      noteId,
      mode,
      durationSeconds,
      pomodoroCycles: options?.pomodoroCycles ?? null,
      actionPath: options?.actionPath ?? null,
      actionArgs: options?.actionArgs ?? null,
      webhookUrl: options?.webhookUrl ?? null,
    }),

  pauseTimer: (noteId: string) => invoke<void>('pause_timer', { noteId }),
  resumeTimer: (noteId: string) => invoke<void>('resume_timer', { noteId }),
  cancelTimer: (noteId: string) => invoke<void>('cancel_timer', { noteId }),

  getTimerState: (noteId: string) =>
    invoke<TimerStatePayload | null>('get_timer_state', { noteId }),

  getSetting: (key: string) => invoke<string | null>('get_setting', { key }),

  setSetting: (key: string, value: string) => invoke<void>('set_setting', { key, value }),

  listSettings: () => invoke<Array<{ key: string; value: string }>>('list_settings'),

  openUrl: (url: string) => invoke<void>('open_url', { url }),

  pomodoroStats: () =>
    invoke<{
      today_seconds: number;
      week_seconds: number;
      last_seven_days: Array<{ date: string; seconds: number }>;
    }>('pomodoro_stats'),

  listShortcutBindings: () =>
    invoke<Array<{ action: string; accelerator: string; default_accelerator: string }>>(
      'list_shortcut_bindings',
    ),
  setShortcut: (action: string, accelerator: string) =>
    invoke<string>('set_shortcut', { action, accelerator }),
  resetShortcut: (action: string) => invoke<string>('reset_shortcut', { action }),

  exportBackup: (path: string) => invoke<void>('export_backup', { path }),
  importBackup: (path: string) => invoke<number>('import_backup', { path }),

  // Google Calendar
  googleIsConfigured: () => invoke<boolean>('google_is_configured'),
  googleConnect: () => invoke<GoogleAccountInfo>('google_connect'),
  googleDisconnect: () => invoke<void>('google_disconnect'),
  googleAccount: () => invoke<GoogleAccountInfo | null>('google_account'),
  googleSync: (
    rangeKind: SyncRangeKind,
    rangeStart: number | null = null,
    rangeEnd: number | null = null,
  ) =>
    invoke<GoogleEventRecord[]>('google_sync', {
      rangeKind,
      rangeStart,
      rangeEnd,
    }),
  googleListEvents: () => invoke<GoogleEventRecord[]>('google_list_events'),
  googleSetEventTimer: (eventId: string, armed: boolean, offsetSeconds: number) =>
    invoke<void>('google_set_event_timer', { eventId, armed, offsetSeconds }),
  googleOpenEvent: (eventId: string) => invoke<void>('google_open_event', { eventId }),
  googleSyncTasks: () => invoke<string>('google_sync_tasks'),

  // Outlook (mirrors the Google command shape).
  outlookIsConfigured: () => invoke<boolean>('outlook_is_configured'),
  outlookConnect: () => invoke<GoogleAccountInfo>('outlook_connect'),
  outlookDisconnect: () => invoke<void>('outlook_disconnect'),
  outlookAccount: () => invoke<GoogleAccountInfo | null>('outlook_account'),
  outlookSync: (
    rangeKind: SyncRangeKind,
    rangeStart: number | null = null,
    rangeEnd: number | null = null,
  ) =>
    invoke<GoogleEventRecord[]>('outlook_sync', {
      rangeKind,
      rangeStart,
      rangeEnd,
    }),
  outlookListEvents: () => invoke<GoogleEventRecord[]>('outlook_list_events'),
};
