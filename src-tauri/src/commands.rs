use crate::google::{self, GoogleAccountInfo, SyncRange};
use crate::launcher;
use crate::outlook;
use crate::shortcuts;
use crate::storage::{GoogleEventRecord, NoteRecord, PomodoroDayBucket, StackRecord, Storage};
use crate::timer::{PostAction, TimerEngine, TimerMode, TimerStateSnapshot};
use crate::window_manager::{WindowManager, SETTING_PRIVACY_HIDE_FROM_CAPTURE};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn create_note(
    initial_position: Option<(f64, f64)>,
    app: AppHandle,
    wm: State<WindowManager>,
) -> Result<NoteRecord, String> {
    wm.create_new_note(&app, initial_position)
}

#[tauri::command]
pub fn update_note_content(
    note_id: String,
    content: String,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .update_content(&note_id, &content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note_color(
    note_id: String,
    color_id: String,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .update_color(&note_id, &color_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note_text_color(
    note_id: String,
    text_color: Option<String>,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .update_text_color(&note_id, text_color.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note_opacity(
    note_id: String,
    opacity: f64,
    app: AppHandle,
    wm: State<WindowManager>,
) -> Result<(), String> {
    wm.set_opacity(&app, &note_id, opacity)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note_position(
    note_id: String,
    x: f64,
    y: f64,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .update_position(&note_id, x, y)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note_size(
    note_id: String,
    width: f64,
    height: f64,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .update_size(&note_id, width, height)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_always_on_top(
    note_id: String,
    app: AppHandle,
    wm: State<WindowManager>,
) -> Result<bool, String> {
    wm.toggle_pin(&app, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(
    note_id: String,
    app: AppHandle,
    wm: State<WindowManager>,
    engine: State<TimerEngine>,
) -> Result<(), String> {
    // Cancel any in-flight timer first so the engine's ticking thread doesn't
    // keep firing events for a note that no longer exists.
    engine.cancel(&note_id);
    wm.delete_note(&app, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note_tags(
    note_id: String,
    tags: Option<String>,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .update_tags(&note_id, tags.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_stacks(storage: State<Storage>) -> Result<Vec<StackRecord>, String> {
    storage.list_stacks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_stack(
    name: String,
    color: Option<String>,
    storage: State<Storage>,
) -> Result<StackRecord, String> {
    storage
        .create_stack(&name, color.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_stack(
    id: String,
    name: String,
    color: Option<String>,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .update_stack(&id, &name, color.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_stack(id: String, storage: State<Storage>) -> Result<(), String> {
    storage.delete_stack(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_note_stack(
    note_id: String,
    stack_id: Option<String>,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .set_note_stack(&note_id, stack_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note_recurring_rule(
    note_id: String,
    rule: Option<String>,
    storage: State<Storage>,
) -> Result<(), String> {
    storage
        .update_recurring_rule(&note_id, rule.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_notes(storage: State<Storage>) -> Result<Vec<NoteRecord>, String> {
    storage.list_notes().map_err(|e| e.to_string())
}

/// Archive a note: hides it from the active list but keeps the row.
/// Cancels any running timer and closes the open window so the user
/// gets immediate visual feedback.
#[tauri::command]
pub fn archive_note(
    note_id: String,
    app: AppHandle,
    storage: State<Storage>,
    wm: State<WindowManager>,
    engine: State<TimerEngine>,
) -> Result<(), String> {
    engine.cancel(&note_id);
    wm.close_note_window(&app, &note_id)
        .map_err(|e| e.to_string())?;
    storage.archive_note(&note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unarchive_note(note_id: String, storage: State<Storage>) -> Result<(), String> {
    storage.unarchive_note(&note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_archived_notes(storage: State<Storage>) -> Result<Vec<NoteRecord>, String> {
    storage.list_archived_notes().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_notes(ordered_ids: Vec<String>, storage: State<Storage>) -> Result<(), String> {
    storage
        .reorder_notes(&ordered_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn focus_note(note_id: String, app: AppHandle, wm: State<WindowManager>) -> Result<(), String> {
    wm.focus_note(&app, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_all_notes(app: AppHandle, wm: State<WindowManager>) -> Result<(), String> {
    wm.hide_all_notes(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn focus_only_note(
    note_id: String,
    app: AppHandle,
    wm: State<WindowManager>,
) -> Result<(), String> {
    wm.focus_only_note(&app, &note_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn show_all_notes(app: AppHandle, wm: State<WindowManager>) -> Result<(), String> {
    wm.show_all_notes(&app).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn start_timer(
    note_id: String,
    mode: String,
    duration_seconds: Option<u64>,
    pomodoro_cycles: Option<u32>,
    action_path: Option<String>,
    action_args: Option<String>,
    webhook_url: Option<String>,
    engine: State<TimerEngine>,
) -> Result<(), String> {
    let m = TimerMode::from_str(&mode).ok_or_else(|| format!("invalid mode: {}", mode))?;
    engine.start(
        note_id,
        m,
        duration_seconds.map(|d| d as i64),
        pomodoro_cycles,
        PostAction {
            path: action_path,
            args: action_args,
        },
        webhook_url.filter(|s| !s.trim().is_empty()),
    );
    Ok(())
}

#[tauri::command]
pub fn pause_timer(note_id: String, engine: State<TimerEngine>) -> Result<(), String> {
    engine.pause(&note_id);
    Ok(())
}

#[tauri::command]
pub fn resume_timer(note_id: String, engine: State<TimerEngine>) -> Result<(), String> {
    engine.resume(&note_id);
    Ok(())
}

#[tauri::command]
pub fn cancel_timer(note_id: String, engine: State<TimerEngine>) -> Result<(), String> {
    engine.cancel(&note_id);
    Ok(())
}

#[tauri::command]
pub fn get_timer_state(
    note_id: String,
    engine: State<TimerEngine>,
) -> Result<Option<TimerStateSnapshot>, String> {
    Ok(engine.snapshot(&note_id))
}

#[derive(Serialize, serde::Deserialize, Clone)]
pub struct SettingPair {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Clone)]
struct SettingChangedPayload<'a> {
    key: &'a str,
    value: &'a str,
}

#[tauri::command]
pub fn get_setting(key: String, storage: State<Storage>) -> Result<Option<String>, String> {
    storage.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(
    key: String,
    value: String,
    app: AppHandle,
    storage: State<Storage>,
    wm: State<WindowManager>,
) -> Result<(), String> {
    storage
        .set_setting(&key, &value)
        .map_err(|e| e.to_string())?;

    // Side-effects for known settings.
    if key == SETTING_PRIVACY_HIDE_FROM_CAPTURE {
        let enabled = value == "true";
        wm.apply_privacy_to_all(&app, enabled);
    }

    let _ = app.emit(
        "settings:changed",
        SettingChangedPayload {
            key: &key,
            value: &value,
        },
    );
    Ok(())
}

/// Open a URL or file in the user's default handler. Used by the
/// in-note "click on a link with ⌘/Ctrl held" feature so notes can act
/// as launch surfaces without us having to bundle a separate plugin.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    launcher::launch(None, Some(&url));
    Ok(())
}

#[derive(Serialize, Clone)]
pub struct PomodoroStats {
    pub today_seconds: i64,
    pub week_seconds: i64,
    pub last_seven_days: Vec<PomodoroDayBucket>,
}

#[derive(Serialize, Clone)]
pub struct ShortcutBinding {
    pub action: String,
    pub accelerator: String,
    pub default_accelerator: String,
}

#[tauri::command]
pub fn list_shortcut_bindings(storage: State<Storage>) -> Result<Vec<ShortcutBinding>, String> {
    let bindings = shortcuts::current_bindings(&storage);
    Ok(bindings
        .into_iter()
        .map(|(action, accelerator)| ShortcutBinding {
            default_accelerator: match action {
                shortcuts::ShortcutAction::NewNote => "CmdOrCtrl+Shift+N".into(),
                shortcuts::ShortcutAction::HideAll => "CmdOrCtrl+Shift+H".into(),
                shortcuts::ShortcutAction::StartTimer => "CmdOrCtrl+Shift+T".into(),
                shortcuts::ShortcutAction::TogglePin => "CmdOrCtrl+Shift+P".into(),
            },
            action: match action {
                shortcuts::ShortcutAction::NewNote => "new_note".into(),
                shortcuts::ShortcutAction::HideAll => "hide_all".into(),
                shortcuts::ShortcutAction::StartTimer => "start_timer".into(),
                shortcuts::ShortcutAction::TogglePin => "toggle_pin".into(),
            },
            accelerator,
        })
        .collect())
}

#[tauri::command]
pub fn set_shortcut(action: String, accelerator: String, app: AppHandle) -> Result<String, String> {
    shortcuts::set_action_accelerator(&app, &action, &accelerator)
}

#[tauri::command]
pub fn reset_shortcut(action: String, app: AppHandle) -> Result<String, String> {
    shortcuts::reset_action(&app, &action)
}

#[tauri::command]
pub fn pomodoro_stats(storage: State<Storage>) -> Result<PomodoroStats, String> {
    let now = chrono::Utc::now();
    let start_of_today = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|d| d.and_utc().timestamp())
        .unwrap_or(0);
    let start_of_week = now.timestamp() - 7 * 86400;
    let today = storage
        .pomodoro_seconds_since(start_of_today)
        .map_err(|e| e.to_string())?;
    let week = storage
        .pomodoro_seconds_since(start_of_week)
        .map_err(|e| e.to_string())?;
    let buckets = storage
        .pomodoro_daily_buckets(7)
        .map_err(|e| e.to_string())?;
    Ok(PomodoroStats {
        today_seconds: today,
        week_seconds: week,
        last_seven_days: buckets,
    })
}

#[derive(Serialize, serde::Deserialize, Clone)]
pub struct BackupSnapshot {
    pub version: u32,
    pub exported_at: i64,
    pub notes: Vec<NoteRecord>,
    pub settings: Vec<SettingPair>,
}

/// Write a JSON snapshot of every note + every setting to `path`. The
/// blob is verbatim restorable via `import_backup`. Google OAuth tokens,
/// pomodoro session history, and timer engine state are intentionally
/// excluded — re-syncing / re-running gives them back.
#[tauri::command]
pub fn export_backup(path: String, storage: State<Storage>) -> Result<(), String> {
    let notes = storage
        .list_all_notes_for_backup()
        .map_err(|e| e.to_string())?;
    let settings = storage
        .list_settings()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(key, value)| SettingPair { key, value })
        .collect();
    let snapshot = BackupSnapshot {
        version: 1,
        exported_at: chrono::Utc::now().timestamp(),
        notes,
        settings,
    };
    let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Restore a JSON snapshot from `path`, replacing the notes table and
/// merging settings. Returns the number of notes restored.
#[tauri::command]
pub fn import_backup(
    path: String,
    app: AppHandle,
    storage: State<Storage>,
) -> Result<usize, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let snapshot: BackupSnapshot = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if snapshot.version != 1 {
        return Err(format!("unsupported backup version: {}", snapshot.version));
    }
    storage
        .replace_notes(&snapshot.notes)
        .map_err(|e| e.to_string())?;
    for SettingPair { key, value } in &snapshot.settings {
        storage.set_setting(key, value).map_err(|e| e.to_string())?;
    }
    let _ = app.emit("backup:imported", snapshot.notes.len());
    Ok(snapshot.notes.len())
}

#[tauri::command]
pub fn list_settings(storage: State<Storage>) -> Result<Vec<SettingPair>, String> {
    let pairs = storage.list_settings().map_err(|e| e.to_string())?;
    Ok(pairs
        .into_iter()
        .map(|(key, value)| SettingPair { key, value })
        .collect())
}

// ---------------- Google Calendar ----------------

#[tauri::command]
pub fn google_is_configured() -> bool {
    google::is_configured()
}

#[tauri::command]
pub async fn google_connect(
    app: AppHandle,
    storage: State<'_, Storage>,
) -> Result<GoogleAccountInfo, String> {
    let info = google::connect(&storage).await.map_err(|e| e.to_string())?;
    let _ = app.emit("google:account-changed", &info);
    Ok(info)
}

#[tauri::command]
pub fn google_disconnect(app: AppHandle, storage: State<Storage>) -> Result<(), String> {
    google::disconnect(&storage).map_err(|e| e.to_string())?;
    let _ = app.emit("google:account-changed", serde_json::Value::Null);
    Ok(())
}

#[tauri::command]
pub fn google_account(storage: State<Storage>) -> Result<Option<GoogleAccountInfo>, String> {
    google::account(&storage).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn google_sync(
    range_kind: String,
    range_start: Option<i64>,
    range_end: Option<i64>,
    app: AppHandle,
    storage: State<'_, Storage>,
    engine: State<'_, TimerEngine>,
) -> Result<Vec<GoogleEventRecord>, String> {
    let range = SyncRange::from_kind(&range_kind, range_start, range_end)
        .ok_or_else(|| format!("invalid sync range: {}", range_kind))?;
    let events = google::sync(&storage, &engine, range)
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit("google:events-synced", &events);
    Ok(events)
}

#[tauri::command]
pub fn google_list_events(storage: State<Storage>) -> Result<Vec<GoogleEventRecord>, String> {
    storage.list_google_events().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn google_set_event_timer(
    event_id: String,
    armed: bool,
    offset_seconds: i64,
    storage: State<Storage>,
    engine: State<TimerEngine>,
) -> Result<(), String> {
    storage
        .set_event_timer(&event_id, armed, offset_seconds)
        .map_err(|e| e.to_string())?;
    if let Ok(Some(ev)) = storage.get_google_event(&event_id) {
        let now = chrono::Utc::now().timestamp();
        google::schedule_event_timer(&engine, &ev, now);
    }
    Ok(())
}

#[tauri::command]
pub async fn google_sync_tasks(
    app: AppHandle,
    storage: State<'_, Storage>,
) -> Result<String, String> {
    let note_id = google::sync_tasks(&storage)
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit("google:tasks-synced", &note_id);
    Ok(note_id)
}

// ---------------- Outlook (mirrors Google's commands) ----------------

#[tauri::command]
pub fn outlook_is_configured() -> bool {
    outlook::is_configured()
}

#[tauri::command]
pub async fn outlook_connect(
    app: AppHandle,
    storage: State<'_, Storage>,
) -> Result<GoogleAccountInfo, String> {
    let info = outlook::connect(&storage)
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit("outlook:account-changed", &info);
    Ok(info)
}

#[tauri::command]
pub fn outlook_disconnect(app: AppHandle, storage: State<Storage>) -> Result<(), String> {
    outlook::disconnect(&storage).map_err(|e| e.to_string())?;
    let _ = app.emit("outlook:account-changed", serde_json::Value::Null);
    Ok(())
}

#[tauri::command]
pub fn outlook_account(storage: State<Storage>) -> Result<Option<GoogleAccountInfo>, String> {
    outlook::account(&storage).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn outlook_sync(
    range_kind: String,
    range_start: Option<i64>,
    range_end: Option<i64>,
    app: AppHandle,
    storage: State<'_, Storage>,
    engine: State<'_, TimerEngine>,
) -> Result<Vec<GoogleEventRecord>, String> {
    let range = SyncRange::from_kind(&range_kind, range_start, range_end)
        .ok_or_else(|| format!("invalid sync range: {}", range_kind))?;
    let events = outlook::sync(&storage, &engine, range)
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit("outlook:events-synced", &events);
    Ok(events)
}

#[tauri::command]
pub fn outlook_list_events(storage: State<Storage>) -> Result<Vec<GoogleEventRecord>, String> {
    storage.list_outlook_events().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn google_open_event(
    event_id: String,
    app: AppHandle,
    storage: State<Storage>,
    wm: State<WindowManager>,
) -> Result<(), String> {
    let ev = storage
        .get_google_event(&event_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("event not found: {}", event_id))?;
    wm.focus_note(&app, &ev.note_id).map_err(|e| e.to_string())
}
