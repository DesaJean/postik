use crate::storage::{NoteRecord, Storage};
use crate::timer::{TimerEngine, TimerMode, TimerStateSnapshot};
use crate::window_manager::WindowManager;
use tauri::{AppHandle, State};

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
) -> Result<(), String> {
    wm.delete_note(&app, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_notes(storage: State<Storage>) -> Result<Vec<NoteRecord>, String> {
    storage.list_notes().map_err(|e| e.to_string())
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
pub fn show_all_notes(app: AppHandle, wm: State<WindowManager>) -> Result<(), String> {
    wm.show_all_notes(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_timer(
    note_id: String,
    mode: String,
    duration_seconds: Option<u64>,
    engine: State<TimerEngine>,
) -> Result<(), String> {
    let m = TimerMode::from_str(&mode).ok_or_else(|| format!("invalid mode: {}", mode))?;
    engine.start(note_id, m, duration_seconds.map(|d| d as i64));
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
