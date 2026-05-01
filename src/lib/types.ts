export type ColorId = 'amber' | 'teal' | 'purple' | 'pink' | 'blue' | 'gray' | 'transparent';

export type TextColorId = 'auto' | 'dark' | 'medium' | 'light' | 'accent';

export type TimerMode = 'countdown' | 'stopwatch' | 'pomodoro';

export type TimerState = 'idle' | 'running' | 'paused' | 'done';

export type PomodoroPhase = 'work' | 'break';

export interface NoteConfig {
  id: string;
  content: string;
  color_id: ColorId;
  opacity: number;
  always_on_top: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
  created_at: number;
  updated_at: number;
  text_color: TextColorId | null;
  /** Set when this note is backed by a Google Calendar event. The note
   * is rendered read-only and its content is synced from Google. */
  event_id: string | null;
}

export interface GoogleAccountInfo {
  email: string;
  connected_at: number;
}

export interface GoogleEventRecord {
  event_id: string;
  note_id: string;
  title: string;
  description: string;
  start_time: number;
  end_time: number;
  html_link: string | null;
  timer_armed: boolean;
  timer_offset_seconds: number;
  synced_at: number;
}

export type SyncRangeKind = 'today' | 'seven_days' | 'custom';

export interface TimerStatePayload {
  note_id: string;
  mode: TimerMode;
  state: TimerState;
  duration_seconds: number | null;
  elapsed_seconds: number;
  remaining_seconds: number | null;
  pomodoro_phase: PomodoroPhase | null;
}

export interface TimerTickPayload {
  note_id: string;
  mode: TimerMode;
  state: TimerState;
  elapsed_seconds: number;
  remaining_seconds: number | null;
  phase: PomodoroPhase | null;
}

export interface TimerDonePayload {
  note_id: string;
  mode: TimerMode;
  phase: PomodoroPhase | null;
}
