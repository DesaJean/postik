use crate::storage::Storage;
use crate::window_manager::WindowManager;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

/// One named action a global shortcut can trigger. The accelerator
/// (e.g. "CmdOrCtrl+Shift+N") for each action is read from settings,
/// falling back to the default below when not customised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    NewNote,
    HideAll,
    StartTimer,
    TogglePin,
}

impl ShortcutAction {
    fn as_str(self) -> &'static str {
        match self {
            ShortcutAction::NewNote => "new_note",
            ShortcutAction::HideAll => "hide_all",
            ShortcutAction::StartTimer => "start_timer",
            ShortcutAction::TogglePin => "toggle_pin",
        }
    }
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "new_note" => Some(Self::NewNote),
            "hide_all" => Some(Self::HideAll),
            "start_timer" => Some(Self::StartTimer),
            "toggle_pin" => Some(Self::TogglePin),
            _ => None,
        }
    }
    fn setting_key(self) -> String {
        format!("shortcut_{}", self.as_str())
    }
    fn default_accelerator(self) -> &'static str {
        match self {
            ShortcutAction::NewNote => "CmdOrCtrl+Shift+N",
            ShortcutAction::HideAll => "CmdOrCtrl+Shift+H",
            ShortcutAction::StartTimer => "CmdOrCtrl+Shift+T",
            ShortcutAction::TogglePin => "CmdOrCtrl+Shift+P",
        }
    }
    pub const ALL: [ShortcutAction; 4] = [
        ShortcutAction::NewNote,
        ShortcutAction::HideAll,
        ShortcutAction::StartTimer,
        ShortcutAction::TogglePin,
    ];
}

/// Reverse map: Shortcut → action. Updated whenever bindings change so
/// the static `handler` fn can dispatch based on what was pressed.
static REGISTRY: OnceLock<Mutex<HashMap<Shortcut, ShortcutAction>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<Shortcut, ShortcutAction>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn parse(accelerator: &str) -> Option<Shortcut> {
    accelerator.parse::<Shortcut>().ok()
}

fn read_accelerator(storage: &Storage, action: ShortcutAction) -> String {
    storage
        .get_setting(&action.setting_key())
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| action.default_accelerator().to_string())
}

/// Returns the (action, accelerator) pairs that are currently in effect.
/// Used by the Settings UI to display the user's bindings.
pub fn current_bindings(storage: &Storage) -> Vec<(ShortcutAction, String)> {
    ShortcutAction::ALL
        .iter()
        .map(|a| (*a, read_accelerator(storage, *a)))
        .collect()
}

/// (Re-)register every action's shortcut. Unregisters anything we
/// previously installed first so toggling never leaves stale handlers.
/// Failures (e.g. a system-reserved combo on Windows) are logged so a
/// single bad binding doesn't break the others.
pub fn register_all(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let storage: tauri::State<Storage> = app.state();
    let gs = app.global_shortcut();

    // Unregister whatever's currently in our registry, then build a
    // fresh one from the current settings.
    {
        let mut reg = registry().lock();
        for shortcut in reg.keys() {
            let _ = gs.unregister(*shortcut);
        }
        reg.clear();
    }

    let mut new_reg = HashMap::new();
    for action in ShortcutAction::ALL {
        let accel = read_accelerator(&storage, action);
        let Some(shortcut) = parse(&accel) else {
            log::warn!(
                "could not parse shortcut accelerator {:?} for {:?}",
                accel,
                action
            );
            continue;
        };
        if let Err(e) = gs.register(shortcut) {
            log::warn!(
                "global shortcut register failed for {:?} ({:?}): {}",
                action,
                accel,
                e
            );
            continue;
        }
        new_reg.insert(shortcut, action);
    }
    *registry().lock() = new_reg;
    Ok(())
}

pub fn handler(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let action = registry().lock().get(shortcut).copied();
    let Some(action) = action else {
        return;
    };
    let wm: tauri::State<WindowManager> = app.state();
    match action {
        ShortcutAction::NewNote => {
            if let Err(e) = wm.create_new_note(app, None) {
                log::warn!("create_new_note failed: {}", e);
            }
            let _ = app.emit("shortcut:new-note", ());
        }
        ShortcutAction::HideAll => {
            // Stealth toggle — see the long-form rationale in the
            // earlier version of this file (kept brief here).
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
        }
        ShortcutAction::StartTimer => {
            if let Some(note_id) = wm.focused_note_id(app) {
                let _ = app.emit_to(format!("note-{}", note_id), "shortcut:start-timer", ());
            }
        }
        ShortcutAction::TogglePin => {
            if let Some(note_id) = wm.focused_note_id(app) {
                let _ = wm.toggle_pin(app, &note_id);
            }
        }
    }
}

/// Persist a new accelerator for an action and re-register all
/// shortcuts so the change takes effect immediately. Returns the
/// stored accelerator (which is `accelerator` on success, or an error
/// describing why the input was rejected).
pub fn set_action_accelerator(
    app: &AppHandle,
    action_id: &str,
    accelerator: &str,
) -> Result<String, String> {
    let action = ShortcutAction::from_str(action_id)
        .ok_or_else(|| format!("unknown action: {action_id}"))?;
    if parse(accelerator).is_none() {
        return Err(format!("invalid accelerator: {accelerator}"));
    }
    let storage: tauri::State<Storage> = app.state();
    storage
        .set_setting(&action.setting_key(), accelerator)
        .map_err(|e| e.to_string())?;
    register_all(app).map_err(|e| e.to_string())?;
    Ok(accelerator.to_string())
}

/// Reset an action to its default accelerator (clears the setting).
pub fn reset_action(app: &AppHandle, action_id: &str) -> Result<String, String> {
    let action = ShortcutAction::from_str(action_id)
        .ok_or_else(|| format!("unknown action: {action_id}"))?;
    let storage: tauri::State<Storage> = app.state();
    storage
        .set_setting(&action.setting_key(), "")
        .map_err(|e| e.to_string())?;
    register_all(app).map_err(|e| e.to_string())?;
    Ok(action.default_accelerator().to_string())
}
