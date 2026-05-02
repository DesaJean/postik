use crate::launcher;
use crate::storage::{Storage, TimerRecord};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

const POMODORO_WORK_SECONDS: i64 = 25 * 60;
const POMODORO_BREAK_SECONDS: i64 = 5 * 60;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TimerMode {
    Countdown,
    Stopwatch,
    Pomodoro,
}

impl TimerMode {
    pub fn as_str(self) -> &'static str {
        match self {
            TimerMode::Countdown => "countdown",
            TimerMode::Stopwatch => "stopwatch",
            TimerMode::Pomodoro => "pomodoro",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "countdown" => Some(Self::Countdown),
            "stopwatch" => Some(Self::Stopwatch),
            "pomodoro" => Some(Self::Pomodoro),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Done,
}

impl TimerState {
    pub fn as_str(self) -> &'static str {
        match self {
            TimerState::Idle => "idle",
            TimerState::Running => "running",
            TimerState::Paused => "paused",
            TimerState::Done => "done",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PomodoroPhase {
    Work,
    Break,
}

impl PomodoroPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            PomodoroPhase::Work => "work",
            PomodoroPhase::Break => "break",
        }
    }
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "work" => Some(Self::Work),
            "break" => Some(Self::Break),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TimerStateSnapshot {
    pub note_id: String,
    pub mode: TimerMode,
    pub state: TimerState,
    pub duration_seconds: Option<i64>,
    pub elapsed_seconds: i64,
    pub remaining_seconds: Option<i64>,
    pub pomodoro_phase: Option<PomodoroPhase>,
}

#[derive(Debug, Clone, Serialize)]
struct TickPayload<'a> {
    note_id: &'a str,
    mode: &'a str,
    state: &'a str,
    elapsed_seconds: i64,
    remaining_seconds: Option<i64>,
    phase: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize)]
struct DonePayload<'a> {
    note_id: &'a str,
    mode: &'a str,
    phase: Option<&'a str>,
}

#[derive(Debug, Clone, Default)]
pub struct PostAction {
    pub path: Option<String>,
    pub args: Option<String>,
}

impl PostAction {
    fn is_empty(&self) -> bool {
        self.path.as_deref().map(str::trim).unwrap_or("").is_empty()
            && self.args.as_deref().map(str::trim).unwrap_or("").is_empty()
    }

    fn fire(&self) {
        if self.is_empty() {
            return;
        }
        launcher::launch(self.path.as_deref(), self.args.as_deref());
    }
}

#[derive(Debug)]
struct LiveTimer {
    mode: TimerMode,
    state: TimerState,
    duration_seconds: Option<i64>,
    elapsed_seconds: i64,
    pomodoro_phase: Option<PomodoroPhase>,
    /// Pomodoro work cycles to run before auto-ending. `None` keeps the
    /// legacy infinite-cycle behaviour for callers that don't supply it.
    pomodoro_total_cycles: Option<u32>,
    /// Work cycles that have completed so far (incremented on Work→Break).
    pomodoro_completed_cycles: u32,
    post_action: PostAction,
}

impl LiveTimer {
    fn snapshot(&self, note_id: &str) -> TimerStateSnapshot {
        let remaining = match self.mode {
            TimerMode::Countdown => self
                .duration_seconds
                .map(|d| (d - self.elapsed_seconds).max(0)),
            TimerMode::Pomodoro => {
                let phase_total = match self.pomodoro_phase.unwrap_or(PomodoroPhase::Work) {
                    PomodoroPhase::Work => POMODORO_WORK_SECONDS,
                    PomodoroPhase::Break => POMODORO_BREAK_SECONDS,
                };
                Some((phase_total - self.elapsed_seconds).max(0))
            }
            TimerMode::Stopwatch => None,
        };
        TimerStateSnapshot {
            note_id: note_id.to_string(),
            mode: self.mode,
            state: self.state,
            duration_seconds: self.duration_seconds,
            elapsed_seconds: self.elapsed_seconds,
            remaining_seconds: remaining,
            pomodoro_phase: self.pomodoro_phase,
        }
    }

    fn to_record(&self, note_id: &str, started_at: Option<i64>) -> TimerRecord {
        TimerRecord {
            note_id: note_id.to_string(),
            mode: self.mode.as_str().into(),
            duration_seconds: self.duration_seconds,
            elapsed_seconds: self.elapsed_seconds,
            state: self.state.as_str().into(),
            pomodoro_phase: self.pomodoro_phase.map(|p| p.as_str().into()),
            started_at,
            post_action_path: self.post_action.path.clone(),
            post_action_args: self.post_action.args.clone(),
            pomodoro_total_cycles: self.pomodoro_total_cycles.map(|n| n as i64),
            pomodoro_completed_cycles: Some(self.pomodoro_completed_cycles as i64),
        }
    }
}

#[derive(Clone)]
pub struct TimerEngine {
    inner: Arc<Mutex<HashMap<String, LiveTimer>>>,
    storage: Storage,
    app: AppHandle,
}

impl TimerEngine {
    pub fn new(app: AppHandle, storage: Storage) -> Self {
        let engine = Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            storage,
            app,
        };
        engine.spawn_tick_loop();
        engine
    }

    pub fn start(
        &self,
        note_id: String,
        mode: TimerMode,
        duration_seconds: Option<i64>,
        pomodoro_total_cycles: Option<u32>,
        post_action: PostAction,
    ) {
        let timer = LiveTimer {
            mode,
            state: TimerState::Running,
            duration_seconds,
            elapsed_seconds: 0,
            pomodoro_phase: if mode == TimerMode::Pomodoro {
                Some(PomodoroPhase::Work)
            } else {
                None
            },
            pomodoro_total_cycles: if mode == TimerMode::Pomodoro {
                pomodoro_total_cycles.filter(|n| *n > 0)
            } else {
                None
            },
            pomodoro_completed_cycles: 0,
            post_action,
        };
        self.persist(&note_id, &timer);
        self.inner.lock().insert(note_id, timer);
    }

    pub fn pause(&self, note_id: &str) {
        let mut map = self.inner.lock();
        if let Some(t) = map.get_mut(note_id) {
            if t.state == TimerState::Running {
                t.state = TimerState::Paused;
                self.persist(note_id, t);
            }
        }
    }

    pub fn resume(&self, note_id: &str) {
        let mut map = self.inner.lock();
        if let Some(t) = map.get_mut(note_id) {
            if t.state == TimerState::Paused {
                t.state = TimerState::Running;
                self.persist(note_id, t);
            }
        }
    }

    pub fn cancel(&self, note_id: &str) {
        // Stopwatches don't auto-end — for them, the user clicking Cancel is
        // the natural "stop", so honour the configured post-action there.
        // For countdown/pomodoro, Cancel means abandon, so we don't fire.
        let (had_timer, action_to_fire) = {
            let mut map = self.inner.lock();
            let action = map.get(note_id).and_then(|t| {
                if t.mode == TimerMode::Stopwatch {
                    Some(t.post_action.clone())
                } else {
                    None
                }
            });
            let existed = map.remove(note_id).is_some();
            (existed, action)
        };
        let _ = self.storage.delete_timer(note_id);
        if let Some(action) = action_to_fire {
            action.fire();
        }
        // Tell every open window the timer is gone so any chime / Done UI
        // bound to this note shuts down. Without this, dismissing the
        // alarm via the Calendar tab's bell — or via auto-sync's implicit
        // cleanup — left an open note window stuck in the "Done" state
        // because nothing told the frontend to refresh.
        if had_timer {
            #[derive(serde::Serialize, Clone)]
            struct CancelledPayload<'a> {
                note_id: &'a str,
            }
            let _ = self
                .app
                .emit("timer:cancelled", CancelledPayload { note_id });
        }
    }

    pub fn snapshot(&self, note_id: &str) -> Option<TimerStateSnapshot> {
        self.inner.lock().get(note_id).map(|t| t.snapshot(note_id))
    }

    fn persist(&self, note_id: &str, t: &LiveTimer) {
        let started_at = Some(chrono::Utc::now().timestamp());
        if let Err(e) = self.storage.upsert_timer(&t.to_record(note_id, started_at)) {
            log::warn!("failed to persist timer for {}: {}", note_id, e);
        }
    }

    fn spawn_tick_loop(&self) {
        let inner = self.inner.clone();
        let app = self.app.clone();
        let storage = self.storage.clone();

        std::thread::spawn(move || {
            let mut last_persist = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let mut to_emit_done: Vec<(String, TimerMode, Option<PomodoroPhase>, String)> =
                    Vec::new();
                let mut actions_to_fire: Vec<PostAction> = Vec::new();
                {
                    let mut map = inner.lock();
                    for (note_id, t) in map.iter_mut() {
                        if t.state != TimerState::Running {
                            continue;
                        }
                        t.elapsed_seconds += 1;

                        let snap = t.snapshot(note_id);
                        let _ = app.emit(
                            "timer:tick",
                            TickPayload {
                                note_id,
                                mode: t.mode.as_str(),
                                state: t.state.as_str(),
                                elapsed_seconds: t.elapsed_seconds,
                                remaining_seconds: snap.remaining_seconds,
                                phase: t.pomodoro_phase.map(|p| p.as_str()),
                            },
                        );

                        match t.mode {
                            TimerMode::Countdown => {
                                if let Some(d) = t.duration_seconds {
                                    if t.elapsed_seconds >= d {
                                        t.state = TimerState::Done;
                                        to_emit_done.push((
                                            note_id.clone(),
                                            t.mode,
                                            None,
                                            preview_for(&storage, note_id),
                                        ));
                                        actions_to_fire.push(t.post_action.clone());
                                    }
                                }
                            }
                            TimerMode::Pomodoro => {
                                let phase = t.pomodoro_phase.unwrap_or(PomodoroPhase::Work);
                                let total = match phase {
                                    PomodoroPhase::Work => POMODORO_WORK_SECONDS,
                                    PomodoroPhase::Break => POMODORO_BREAK_SECONDS,
                                };
                                if t.elapsed_seconds >= total {
                                    // A work-session boundary counts toward the
                                    // configured cycle limit; break boundaries
                                    // don't.
                                    if phase == PomodoroPhase::Work {
                                        t.pomodoro_completed_cycles =
                                            t.pomodoro_completed_cycles.saturating_add(1);
                                    }
                                    let limit_reached = t
                                        .pomodoro_total_cycles
                                        .map(|n| t.pomodoro_completed_cycles >= n)
                                        .unwrap_or(false);
                                    to_emit_done.push((
                                        note_id.clone(),
                                        t.mode,
                                        Some(phase),
                                        preview_for(&storage, note_id),
                                    ));
                                    if limit_reached {
                                        // Stop cycling; let the frontend show
                                        // Done and surface a Dismiss button.
                                        t.state = TimerState::Done;
                                        actions_to_fire.push(t.post_action.clone());
                                    } else {
                                        let next = match phase {
                                            PomodoroPhase::Work => PomodoroPhase::Break,
                                            PomodoroPhase::Break => PomodoroPhase::Work,
                                        };
                                        t.pomodoro_phase = Some(next);
                                        t.elapsed_seconds = 0;
                                    }
                                }
                            }
                            TimerMode::Stopwatch => {}
                        }
                    }

                    // Persist every 10 seconds rather than every tick (reduces I/O).
                    if last_persist.elapsed() >= std::time::Duration::from_secs(10) {
                        for (note_id, t) in map.iter() {
                            if t.state == TimerState::Running {
                                let _ = storage.upsert_timer(
                                    &t.to_record(note_id, Some(chrono::Utc::now().timestamp())),
                                );
                            }
                        }
                        last_persist = std::time::Instant::now();
                    }
                }

                for (note_id, mode, phase, preview) in to_emit_done {
                    let _ = app.emit(
                        "timer:done",
                        DonePayload {
                            note_id: &note_id,
                            mode: mode.as_str(),
                            phase: phase.map(|p| p.as_str()),
                        },
                    );
                    fire_notification(&app, &preview, mode, phase);
                }
                for action in actions_to_fire {
                    action.fire();
                }
            }
        });
    }
}

fn preview_for(storage: &Storage, note_id: &str) -> String {
    storage
        .get_note(note_id)
        .ok()
        .flatten()
        .map(|n| n.content.chars().take(30).collect::<String>())
        .unwrap_or_default()
}

fn fire_notification(
    app: &AppHandle,
    preview: &str,
    mode: TimerMode,
    phase: Option<PomodoroPhase>,
) {
    let title = match (mode, phase) {
        (TimerMode::Pomodoro, Some(PomodoroPhase::Work)) => {
            "🍅 Work session done — break time".to_string()
        }
        (TimerMode::Pomodoro, Some(PomodoroPhase::Break)) => {
            "🍅 Break done — back to work".to_string()
        }
        _ => "⏱ Timer done".to_string(),
    };
    let body = if preview.is_empty() {
        "(empty note)".to_string()
    } else {
        preview.to_string()
    };
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        log::warn!("notification failed: {}", e);
    }
}
