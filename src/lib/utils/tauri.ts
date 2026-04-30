import { invoke } from '@tauri-apps/api/core';
import type { ColorId, NoteConfig, TextColorId, TimerMode, TimerStatePayload } from '../types';

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

  focusNote: (noteId: string) => invoke<void>('focus_note', { noteId }),

  hideAllNotes: () => invoke<void>('hide_all_notes'),
  showAllNotes: () => invoke<void>('show_all_notes'),

  startTimer: (noteId: string, mode: TimerMode, durationSeconds: number | null) =>
    invoke<void>('start_timer', { noteId, mode, durationSeconds }),

  pauseTimer: (noteId: string) => invoke<void>('pause_timer', { noteId }),
  resumeTimer: (noteId: string) => invoke<void>('resume_timer', { noteId }),
  cancelTimer: (noteId: string) => invoke<void>('cancel_timer', { noteId }),

  getTimerState: (noteId: string) =>
    invoke<TimerStatePayload | null>('get_timer_state', { noteId }),

  getSetting: (key: string) => invoke<string | null>('get_setting', { key }),

  setSetting: (key: string, value: string) => invoke<void>('set_setting', { key, value }),

  listSettings: () => invoke<Array<{ key: string; value: string }>>('list_settings'),
};
