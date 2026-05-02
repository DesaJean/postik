use crate::google::{self, GoogleAccountInfo, SyncRange};
use crate::launcher;
use crate::storage::{GoogleEventRecord, NoteRecord, Storage};
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
pub fn list_notes(storage: State<Storage>) -> Result<Vec<NoteRecord>, String> {
    storage.list_notes().map_err(|e| e.to_string())
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

#[tauri::command]
pub fn start_timer(
    note_id: String,
    mode: String,
    duration_seconds: Option<u64>,
    pomodoro_cycles: Option<u32>,
    action_path: Option<String>,
    action_args: Option<String>,
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

#[derive(Serialize, Clone)]
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
