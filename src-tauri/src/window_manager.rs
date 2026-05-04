use crate::storage::{NoteRecord, Storage};
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
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

pub const SETTING_PRIVACY_HIDE_FROM_CAPTURE: &str = "privacy_hide_from_capture";

/// Read a boolean setting from storage. Returns `default` if the row is
/// missing or the value can't be parsed — callers don't need to distinguish.
///
/// Gated to non-Windows: the only callers (the privacy/content-protection
/// branches in `create_controller` / `open_window_for`) are themselves
/// disabled on Windows, so this function would be dead code there and
/// the release `deny(warnings)` lint would reject the build.
#[cfg(not(target_os = "windows"))]
fn read_bool_setting(storage: &Storage, key: &str, default: bool) -> bool {
    storage
        .get_setting(key)
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(default)
}

#[derive(Clone)]
pub struct WindowManager {
    storage: Storage,
    open_labels: Arc<Mutex<HashSet<String>>>,
    /// Stealth mode: when true, the tray icon, controller, and all note
    /// windows are hidden so nothing about Postik is visible during a screen
    /// share. Toggled by the global ⌘⇧H shortcut.
    stealth: Arc<AtomicBool>,
}

impl WindowManager {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            open_labels: Arc::new(Mutex::new(HashSet::new())),
            stealth: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_stealth(&self) -> bool {
        self.stealth.load(Ordering::Relaxed)
    }

    pub fn set_stealth(&self, value: bool) {
        self.stealth.store(value, Ordering::Relaxed);
    }

    /// Build the controller window. Visible on launch so the user gets immediate
    /// feedback; subsequent visibility is toggled via tray icon and close button.
    pub fn create_controller(&self, app: &AppHandle) -> tauri::Result<()> {
        let builder =
            WebviewWindowBuilder::new(app, CONTROLLER_LABEL, WebviewUrl::App("index.html".into()))
                .title("Postik")
                .inner_size(360.0, 480.0)
                .min_inner_size(320.0, 360.0)
                .resizable(true)
                .visible(true)
                .skip_taskbar(true);
        // Content protection is a macOS-focused feature; on Windows the
        // WDA_EXCLUDEFROMCAPTURE flag has been observed to interfere with
        // WebView2 rendering. Shadow `builder` on non-Windows to apply it
        // without needing `let mut` (which fails the release `deny(warnings)`
        // lint on Windows where the `mut` would be unused).
        #[cfg(not(target_os = "windows"))]
        let builder = {
            let privacy = read_bool_setting(&self.storage, SETTING_PRIVACY_HIDE_FROM_CAPTURE, true);
            builder.content_protected(privacy)
        };
        let win = builder.build()?;
        // The controller can be closed without quitting the app — intercept the
        // close request and hide the window instead.
        //
        // We also raise the controller's z-level when it's focused so it can
        // come above pinned notes (which sit at the floating level via
        // `always_on_top`). Without this, pinned notes always render on top
        // of the controller regardless of which window the user clicked,
        // because macOS resolves window levels before focus order. Toggling
        // the always-on-top flag on focus/blur lets the focused window win.
        let label = win.label().to_string();
        let app_clone = app.clone();
        win.on_window_event(move |event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if let Some(w) = app_clone.get_webview_window(&label) {
                    let _ = w.hide();
                    api.prevent_close();
                }
            }
            tauri::WindowEvent::Focused(focused) => {
                if let Some(w) = app_clone.get_webview_window(&label) {
                    let _ = w.set_always_on_top(*focused);
                }
            }
            _ => {}
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
        let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
            .title("")
            .inner_size(note.width, note.height)
            .min_inner_size(NOTE_MIN_WIDTH, NOTE_MIN_HEIGHT)
            .position(x, y)
            .decorations(false)
            .always_on_top(note.always_on_top)
            .resizable(true)
            .skip_taskbar(true);

        // Transparent webview windows are unreliable on Windows: WebView2's
        // composition often falls back to a solid white background when the
        // underlying transparency fails, leaving the note rendering blank.
        // Win11 rounds frameless windows at the OS level, so we don't lose
        // the visual on Windows by keeping them opaque. Likewise content
        // protection is a macOS-focused feature; skip it on Windows where
        // the WDA flag has been observed to interfere with rendering.
        // Shadowing builder via `#[cfg]` keeps both branches free of an
        // unused `mut` (which would fail the release `deny(warnings)` lint
        // on the platform that doesn't reassign).
        #[cfg(not(target_os = "windows"))]
        let builder = {
            let privacy = read_bool_setting(&self.storage, SETTING_PRIVACY_HIDE_FROM_CAPTURE, true);
            builder.transparent(true).content_protected(privacy)
        };

        let win = builder.build()?;

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

    /// Hide every note window except the one for `note_id`. Used by the
    /// "focus this note" button in the title bar — the user wants a clean
    /// workspace except for the current task. `show_all_notes` reverses it.
    pub fn focus_only_note(&self, app: &AppHandle, note_id: &str) -> tauri::Result<()> {
        let keep = Self::note_label(note_id);
        for w in app.webview_windows().values() {
            if w.label().starts_with("note-") && w.label() != keep {
                w.hide()?;
            }
        }
        Ok(())
    }

    /// Apply the privacy (content-protection) setting to every existing
    /// window. Called when the user flips the toggle in Settings — runtime
    /// changes don't require recreating the windows.
    pub fn apply_privacy_to_all(&self, app: &AppHandle, enabled: bool) {
        for w in app.webview_windows().values() {
            if let Err(e) = w.set_content_protected(enabled) {
                log::warn!(
                    "set_content_protected({}) failed for {}: {}",
                    enabled,
                    w.label(),
                    e
                );
            }
        }
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

    /// Tile every visible note window into a grid on the primary monitor,
    /// grouped by stack. Stacks are laid out top-to-bottom in their stored
    /// `sort_index` order; unstacked notes go last. Within a stack, notes
    /// flow left-to-right and wrap at the screen's right edge. A larger
    /// vertical gap separates groups so the user can see at a glance which
    /// stack is which.
    ///
    /// Each note's existing width/height is preserved — we only move them.
    /// Both the live window and the storage row are updated so the layout
    /// survives a restart.
    pub fn arrange_notes(&self, app: &AppHandle) -> tauri::Result<()> {
        use std::collections::HashMap;

        let notes = match self.storage.list_notes() {
            Ok(v) => v,
            Err(e) => {
                log::warn!("arrange_notes: list_notes failed: {}", e);
                return Ok(());
            }
        };
        let stacks = self.storage.list_stacks().unwrap_or_default();

        // Bucket notes by their stack_id (None for unstacked).
        let mut by_stack: HashMap<Option<String>, Vec<NoteRecord>> = HashMap::new();
        for n in notes {
            by_stack.entry(n.stack_id.clone()).or_default().push(n);
        }

        // Build the visit order: declared stacks in their sort order, then
        // unstacked, then any orphan stack ids that didn't match (shouldn't
        // happen but cheap to guard).
        let mut groups: Vec<Vec<NoteRecord>> = Vec::new();
        for s in &stacks {
            if let Some(g) = by_stack.remove(&Some(s.id.clone())) {
                groups.push(g);
            }
        }
        if let Some(g) = by_stack.remove(&None) {
            groups.push(g);
        }
        for (_, g) in by_stack.drain() {
            groups.push(g);
        }

        // Logical screen size (set_position with LogicalPosition handles HiDPI).
        let (screen_w, _screen_h) = match app.primary_monitor() {
            Ok(Some(m)) => {
                let scale = m.scale_factor();
                let s = m.size();
                (s.width as f64 / scale, s.height as f64 / scale)
            }
            _ => (1440.0, 900.0),
        };
        let margin: f64 = 24.0;
        let gap: f64 = 12.0;
        let group_gap: f64 = 28.0;
        let max_x = screen_w - margin;

        let mut cur_y = margin;
        for group in groups {
            if group.is_empty() {
                continue;
            }
            let mut row_x = margin;
            let mut row_max_h: f64 = 0.0;

            for note in &group {
                let w = note.width.max(NOTE_MIN_WIDTH);
                let h = note.height.max(NOTE_MIN_HEIGHT);

                // Wrap to the next row when the next tile wouldn't fit.
                if row_x + w > max_x && row_x > margin {
                    cur_y += row_max_h + gap;
                    row_x = margin;
                    row_max_h = 0.0;
                }

                let x = row_x;
                let y = cur_y;

                let label = Self::note_label(&note.id);
                if let Some(win) = app.get_webview_window(&label) {
                    let _ = win.set_position(LogicalPosition::new(x, y));
                }
                let _ = self.storage.update_position(&note.id, x, y);

                row_x += w + gap;
                row_max_h = row_max_h.max(h);
            }

            cur_y += row_max_h + group_gap;
        }
        Ok(())
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
