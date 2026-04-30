export type ColorId = 'amber' | 'teal' | 'purple' | 'pink' | 'blue' | 'gray' | 'transparent';

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
}

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
