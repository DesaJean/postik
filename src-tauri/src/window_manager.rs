use crate::storage::{NoteRecord, Storage};
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, WebviewUrl,
    WebviewWindowBuilder,
};

pub const NOTE_DEFAULT_WIDTH: f64 = 240.0;
pub const NOTE_DEFAULT_HEIGHT: f64 = 200.0;
pub const NOTE_MIN_WIDTH: f64 = 180.0;
pub const NOTE_MIN_HEIGHT: f64 = 140.0;

pub const CONTROLLER_LABEL: &str = "controller";

#[derive(Clone)]
pub struct WindowManager {
    storage: Storage,
    open_labels: Arc<Mutex<HashSet<String>>>,
}

impl WindowManager {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            open_labels: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Build the controller window. Visible on launch so the user gets immediate
    /// feedback; subsequent visibility is toggled via tray icon and close button.
    pub fn create_controller(&self, app: &AppHandle) -> tauri::Result<()> {
        let win = WebviewWindowBuilder::new(
            app,
            CONTROLLER_LABEL,
            WebviewUrl::App("index.html".into()),
        )
        .title("Postik")
        .inner_size(360.0, 480.0)
        .min_inner_size(320.0, 360.0)
        .resizable(true)
        .visible(true)
        .build()?;
        // The controller can be closed without quitting the app — intercept the
        // close request and hide the window instead.
        let label = win.label().to_string();
        let app_clone = app.clone();
        win.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(w) = app_clone.get_webview_window(&label) {
                    let _ = w.hide();
                    api.prevent_close();
                }
            }
        });
        Ok(())
    }

    pub fn show_controller(&self, app: &AppHandle) -> tauri::Result<()> {
        if let Some(w) = app.get_webview_window(CONTROLLER_LABEL) {
            w.show()?;
            w.set_focus()?;
        }
        Ok(())
    }

    pub fn toggle_controller(&self, app: &AppHandle) -> tauri::Result<()> {
        if let Some(w) = app.get_webview_window(CONTROLLER_LABEL) {
            if w.is_visible().unwrap_or(false) {
                w.hide()?;
            } else {
                w.show()?;
                w.set_focus()?;
            }
        }
        Ok(())
    }

    fn note_label(id: &str) -> String {
        format!("note-{}", id)
    }

    /// Compute a fresh position that doesn't perfectly overlap existing windows.
    fn next_position(&self, app: &AppHandle, suggested: Option<(f64, f64)>) -> (f64, f64) {
        let (mut x, mut y) = suggested.unwrap_or((120.0, 120.0));
        let existing = app.webview_windows();
        loop {
            let collision = existing.values().any(|w| {
                if let Ok(pos) = w.outer_position() {
                    let scale = w.scale_factor().unwrap_or(1.0);
                    let lx = pos.x as f64 / scale;
                    let ly = pos.y as f64 / scale;
                    (lx - x).abs() < 1.0 && (ly - y).abs() < 1.0
                } else {
                    false
                }
            });
            if !collision {
                return (x, y);
            }
            x += 20.0;
            y += 20.0;
        }
    }

    /// Reposition a note onto the primary monitor if its persisted coords are off-screen.
    fn clamp_to_monitor(&self, app: &AppHandle, x: f64, y: f64) -> (f64, f64) {
        if let Ok(Some(monitor)) = app.primary_monitor() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let max_x = (size.width as f64 / scale) - NOTE_MIN_WIDTH;
            let max_y = (size.height as f64 / scale) - NOTE_MIN_HEIGHT;
            let nx = x.clamp(0.0, max_x.max(0.0));
            let ny = y.clamp(0.0, max_y.max(0.0));
            return (nx, ny);
        }
        (x, y)
    }

    pub fn create_new_note(
        &self,
        app: &AppHandle,
        suggested_position: Option<(f64, f64)>,
    ) -> Result<NoteRecord, String> {
        let (x, y) = self.next_position(app, suggested_position);
        let record = self
            .storage
            .create_note(x, y, NOTE_DEFAULT_WIDTH, NOTE_DEFAULT_HEIGHT)
            .map_err(|e| e.to_string())?;
        self.open_window_for(app, &record)
            .map_err(|e| e.to_string())?;
        Ok(record)
    }

    pub fn open_window_for(&self, app: &AppHandle, note: &NoteRecord) -> tauri::Result<()> {
        let label = Self::note_label(&note.id);
        if app.get_webview_window(&label).is_some() {
            return Ok(());
        }
        let (x, y) = self.clamp_to_monitor(app, note.x, note.y);

        let url = format!("note.html?id={}", note.id);
        let win = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title("")
            .inner_size(note.width, note.height)
            .min_inner_size(NOTE_MIN_WIDTH, NOTE_MIN_HEIGHT)
            .position(x, y)
            .decorations(false)
            .transparent(true)
            .always_on_top(note.always_on_top)
            .resizable(true)
            .skip_taskbar(false)
            .build()?;

        self.open_labels.lock().insert(label);

        // Persist position/size changes back to SQLite so the note lays itself
        // out the same way next launch.
        let storage = self.storage.clone();
        let note_id = note.id.clone();
        win.on_window_event(move |event| match event {
            tauri::WindowEvent::Moved(PhysicalPosition { x, y }) => {
                let _ = storage.update_position(&note_id, *x as f64, *y as f64);
            }
            tauri::WindowEvent::Resized(size) => {
                let _ = storage.update_size(&note_id, size.width as f64, size.height as f64);
            }
            _ => {}
        });
        Ok(())
    }

    pub fn focus_note(&self, app: &AppHandle, note_id: &str) -> tauri::Result<()> {
        let label = Self::note_label(note_id);
        if let Some(w) = app.get_webview_window(&label) {
            w.show()?;
            w.set_focus()?;
        } else if let Some(rec) = self.storage.get_note(note_id).ok().flatten() {
            self.open_window_for(app, &rec)?;
        }
        Ok(())
    }

    pub fn toggle_pin(&self, app: &AppHandle, note_id: &str) -> tauri::Result<bool> {
        let label = Self::note_label(note_id);
        let Some(w) = app.get_webview_window(&label) else {
            return Ok(false);
        };
        let current = self
            .storage
            .get_note(note_id)
            .ok()
            .flatten()
            .map(|r| r.always_on_top)
            .unwrap_or(true);
        let next = !current;
        w.set_always_on_top(next)?;
        let _ = self.storage.update_always_on_top(note_id, next);
        Ok(next)
    }

    pub fn set_opacity(&self, app: &AppHandle, note_id: &str, opacity: f64) -> tauri::Result<()> {
        let label = Self::note_label(note_id);
        if let Some(_w) = app.get_webview_window(&label) {
            // Opacity is rendered inside the webview (CSS) — no native call needed.
        }
        let _ = self.storage.update_opacity(note_id, opacity);
        Ok(())
    }

    pub fn close_note_window(&self, app: &AppHandle, note_id: &str) -> tauri::Result<()> {
        let label = Self::note_label(note_id);
        if let Some(w) = app.get_webview_window(&label) {
            w.close()?;
        }
        self.open_labels.lock().remove(&label);
        Ok(())
    }

    pub fn delete_note(&self, app: &AppHandle, note_id: &str) -> tauri::Result<()> {
        self.close_note_window(app, note_id)?;
        let _ = self.storage.delete_note(note_id);
        Ok(())
    }

    pub fn hide_all_notes(&self, app: &AppHandle) -> tauri::Result<()> {
        for w in app.webview_windows().values() {
            if w.label().starts_with("note-") {
                w.hide()?;
            }
        }
        Ok(())
    }

    pub fn show_all_notes(&self, app: &AppHandle) -> tauri::Result<()> {
        for w in app.webview_windows().values() {
            if w.label().starts_with("note-") {
                w.show()?;
            }
        }
        Ok(())
    }

    pub fn restore_persisted(&self, app: &AppHandle) -> tauri::Result<()> {
        match self.storage.list_notes() {
            Ok(notes) => {
                for n in notes {
                    if let Err(e) = self.open_window_for(app, &n) {
                        log::warn!("failed to restore window for {}: {}", n.id, e);
                    }
                }
                Ok(())
            }
            Err(e) => {
                log::error!("failed to list notes on startup: {}", e);
                Ok(())
            }
        }
    }

    pub fn focused_note_id(&self, app: &AppHandle) -> Option<String> {
        for w in app.webview_windows().values() {
            if w.is_focused().unwrap_or(false) {
                let label = w.label();
                if let Some(id) = label.strip_prefix("note-") {
                    return Some(id.to_string());
                }
            }
        }
        None
    }
}

/// Helpers for the controller window's logical-position commands.
#[allow(dead_code)]
pub fn logical(x: f64, y: f64) -> LogicalPosition<f64> {
    LogicalPosition::new(x, y)
}

#[allow(dead_code)]
pub fn logical_size(w: f64, h: f64) -> LogicalSize<f64> {
    LogicalSize::new(w, h)
}
