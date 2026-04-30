use crate::window_manager::WindowManager;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent, ShortcutState,
};

pub fn register_all(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let gs = app.global_shortcut();
    let combos = [
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyN),
        Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyN),
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyH),
        Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyH),
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyT),
        Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT),
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyP),
        Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyP),
    ];
    // Register each shortcut individually and log failures instead of bailing.
    // Why: on Windows, a Win+Shift+X combo can collide with a system shortcut
    // and `register` returns Err — propagating that aborted app setup, which
    // looked to users like the app launching and immediately closing.
    for combo in combos {
        if let Err(e) = gs.register(combo) {
            log::warn!("global shortcut register failed ({:?}): {}", combo, e);
        }
    }
    Ok(())
}

pub fn handler(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let wm: tauri::State<WindowManager> = app.state();

    let is_n = shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyN)
        || shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyN);
    let is_h = shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyH)
        || shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyH);
    let is_t = shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyT)
        || shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyT);
    let is_p = shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyP)
        || shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyP);

    if is_n {
        if let Err(e) = wm.create_new_note(app, None) {
            log::warn!("create_new_note failed: {}", e);
        }
        let _ = app.emit("shortcut:new-note", ());
    } else if is_h {
        // Toggle tray-icon visibility. The notes and controller are already
        // excluded from screen capture by content_protected(true), so the
        // tray icon (which lives in the macOS menu bar, outside any Tauri
        // window) is the only Postik element that leaks into a screen share.
        // Hiding it lets the user keep reading their notes while presenting.
        let tray = app.tray_by_id("postik-tray");
        if wm.is_stealth() {
            wm.set_stealth(false);
            if let Some(t) = &tray {
                let _ = t.set_visible(true);
            }
        } else {
            wm.set_stealth(true);
            if let Some(t) = &tray {
                let _ = t.set_visible(false);
            }
        }
    } else if is_t {
        if let Some(note_id) = wm.focused_note_id(app) {
            let _ = app.emit_to(format!("note-{}", note_id), "shortcut:start-timer", ());
        }
    } else if is_p {
        if let Some(note_id) = wm.focused_note_id(app) {
            let _ = wm.toggle_pin(app, &note_id);
        }
    }
}
