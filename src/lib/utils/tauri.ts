import { invoke } from '@tauri-apps/api/core';
import type {
  ColorId,
  GoogleAccountInfo,
  GoogleEventRecord,
  NoteConfig,
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

  updateNoteOpacity: (noteId: string, opacity: number) =>
    invoke<void>('update_note_opacity', { noteId, opacity }),

  updateNotePosition: (noteId: string, x: number, y: number) =>
    invoke<void>('update_note_position', { noteId, x, y }),

  updateNoteSize: (noteId: string, width: number, height: number) =>
    invoke<void>('update_note_size', { noteId, width, height }),

  toggleAlwaysOnTop: (noteId: string) => invoke<boolean>('toggle_always_on_top', { noteId }),

  deleteNote: (noteId: string) => invoke<void>('delete_note', { noteId }),

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
    },
  ) =>
    invoke<void>('start_timer', {
      noteId,
      mode,
      durationSeconds,
      pomodoroCycles: options?.pomodoroCycles ?? null,
      actionPath: options?.actionPath ?? null,
      actionArgs: options?.actionArgs ?? null,
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
};
