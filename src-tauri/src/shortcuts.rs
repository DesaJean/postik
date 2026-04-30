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
    for combo in combos {
        gs.register(combo)?;
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
        // Stealth toggle: hide notes + controller + tray when entering, show
        // them when leaving. Global shortcut works regardless of tray
        // visibility, so the user can always come back from stealth.
        let tray = app.tray_by_id("postik-tray");
        if wm.is_stealth() {
            wm.set_stealth(false);
            if let Some(t) = &tray {
                let _ = t.set_visible(true);
            }
            let _ = wm.show_all_notes(app);
            let _ = app.emit("shortcut:show-all", ());
        } else {
            wm.set_stealth(true);
            let _ = wm.hide_all_notes(app);
            if let Some(c) = app.get_webview_window("controller") {
                let _ = c.hide();
            }
            if let Some(t) = &tray {
                let _ = t.set_visible(false);
            }
            let _ = app.emit("shortcut:hide-all", ());
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
