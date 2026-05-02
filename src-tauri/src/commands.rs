use crate::google::{self, GoogleAccountInfo, SyncRange};
use crate::launcher;
use crate::outlook;
use crate::shortcuts;
use crate::storage::{GoogleEventRecord, NoteRecord, PomodoroDayBucket, StackRecord, Storage};
use crate::timer::{PostAction, TimerEngine, TimerMode, TimerStateSnapshot};
use crate::window_manager::{WindowManager, SETTING_PRIVACY_HIDE_FROM_CAPTURE};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

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
///
/// During an active pomodoro work session, hosts the user has marked
/// as "blocked" return a special error so the frontend can show an
/// override prompt rather than opening silently.
#[tauri::command]
pub fn open_url(
    url: String,
    storage: State<Storage>,
    engine: State<TimerEngine>,
) -> Result<(), String> {
    if engine.is_any_pomodoro_in_work() {
        if let Some(host) = host_of(&url) {
            let blocked = storage
                .get_setting("focus_blocked_hosts")
                .ok()
                .flatten()
                .unwrap_or_default();
            if blocked
                .lines()
                .map(|l| l.trim().to_lowercase())
                .filter(|l| !l.is_empty())
                .any(|pattern| host.contains(&pattern))
            {
                return Err(format!("blocked_during_focus:{host}"));
            }
        }
    }
    launcher::launch(None, Some(&url));
    Ok(())
}

/// Bypass the distraction blocker. Frontend calls this after the user
/// confirms they want to open the URL anyway.
#[tauri::command]
pub fn open_url_force(url: String) -> Result<(), String> {
    launcher::launch(None, Some(&url));
    Ok(())
}

fn host_of(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_lowercase()))
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

/// Where Postik is currently reading/writing its SQLite file. Used by
/// Settings → Storage to show the user what's in effect.
/// Summarize a note's content via the Anthropic API. The user
/// supplies their own API key in Settings — Postik never ships one.
/// Uses Haiku for low cost and fast turnaround on short notes.
#[tauri::command]
pub async fn summarize_note(
    content: String,
    storage: State<'_, Storage>,
) -> Result<String, String> {
    let api_key = storage
        .get_setting("anthropic_api_key")
        .map_err(|e| e.to_string())?
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "No Anthropic API key configured. Set one in Settings → AI.".to_string())?;
    if content.trim().is_empty() {
        return Ok(String::new());
    }
    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 400,
        "messages": [
            {
                "role": "user",
                "content": format!(
                    "Summarize the following sticky note in 1-2 short sentences. Be concise; preserve concrete names, dates, and action items. Output the summary only — no preamble.\n\n---\n{}",
                    content
                ),
            }
        ],
    });
    let resp: serde_json::Value = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    // Successful: { "content": [{"type":"text","text":"…"}], ... }
    // Error: { "error": {"message":"…"}, ... }
    if let Some(err) = resp
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
    {
        return Err(err.to_string());
    }
    let summary = resp
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| {
            arr.iter()
                .find_map(|p| p.get("text").and_then(|t| t.as_str()))
        })
        .unwrap_or("(no response)")
        .trim()
        .to_string();
    Ok(summary)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OrganizeSuggestion {
    pub note_id: String,
    pub tags: Option<String>,
    pub stack_id: Option<String>,
}

/// Ask Claude to read short content snippets for each note and propose
/// `tags` (comma-separated) and an optional `stack_id` chosen from the
/// supplied stack list. The frontend shows the suggestions for review
/// before applying — Postik never writes back without an explicit
/// confirm. The model only sees content + stack metadata; nothing
/// leaves Postik unless the user has set their API key.
#[tauri::command]
pub async fn ai_organize_notes(
    storage: State<'_, Storage>,
) -> Result<Vec<OrganizeSuggestion>, String> {
    let api_key = storage
        .get_setting("anthropic_api_key")
        .map_err(|e| e.to_string())?
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "No Anthropic API key configured. Set one in Settings → AI.".to_string())?;

    let notes = storage.list_notes().map_err(|e| e.to_string())?;
    let stacks = storage.list_stacks().map_err(|e| e.to_string())?;
    let notes_payload: Vec<_> = notes
        .iter()
        .filter(|n| !n.content.trim().is_empty())
        .map(|n| {
            // Cap each note's content sent to the model — long notes
            // bloat tokens and hurt suggestion quality.
            let snippet: String = n.content.chars().take(400).collect();
            serde_json::json!({ "id": n.id, "content": snippet })
        })
        .collect();
    if notes_payload.is_empty() {
        return Ok(vec![]);
    }
    let stacks_payload: Vec<_> = stacks
        .iter()
        .map(|s| serde_json::json!({ "id": s.id, "name": s.name }))
        .collect();

    let prompt = format!(
        "You are organizing a user's sticky notes. For each note, propose 1–3 short lowercase tags (comma-separated, no '#') and optionally assign one of the existing stacks (by id). Skip a stack if none fit.\n\nReturn ONLY valid JSON of the shape: [{{\"note_id\":\"...\",\"tags\":\"a, b\",\"stack_id\":\"... or null\"}}]. No markdown, no explanation.\n\nStacks: {}\n\nNotes: {}",
        serde_json::to_string(&stacks_payload).unwrap_or_default(),
        serde_json::to_string(&notes_payload).unwrap_or_default(),
    );

    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 2000,
        "messages": [{ "role": "user", "content": prompt }],
    });
    let resp: serde_json::Value = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|e| e.to_string())?
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if let Some(err) = resp
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
    {
        return Err(err.to_string());
    }
    let text = resp
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| {
            arr.iter()
                .find_map(|p| p.get("text").and_then(|t| t.as_str()))
        })
        .unwrap_or("[]")
        .trim()
        .to_string();

    // The model occasionally wraps JSON in ```json fences despite our
    // instructions — strip them tolerantly before parsing.
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let parsed: Vec<OrganizeSuggestion> = serde_json::from_str(cleaned).map_err(|e| {
        format!(
            "AI returned non-JSON output ({}). Raw response: {}",
            e, cleaned
        )
    })?;
    Ok(parsed)
}

/// Bulk-apply organize suggestions: writes each suggestion's tags and
/// stack assignment for the matching note in a single call. Returns
/// the count actually applied so the frontend can display feedback.
#[tauri::command]
pub fn apply_organize_suggestions(
    suggestions: Vec<OrganizeSuggestion>,
    storage: State<Storage>,
) -> Result<usize, String> {
    let mut count = 0;
    for s in suggestions {
        if let Some(tags) = &s.tags {
            storage
                .update_tags(&s.note_id, Some(tags))
                .map_err(|e| e.to_string())?;
        }
        if let Some(stack_id) = &s.stack_id {
            storage
                .set_note_stack(&s.note_id, Some(stack_id))
                .map_err(|e| e.to_string())?;
        }
        count += 1;
    }
    Ok(count)
}

/// Suggest a timer duration (in seconds) for a note based on its
/// content. Lightweight prompt that returns just an integer; the
/// frontend pre-fills the duration field with the result. Skipped
/// silently when no API key is set or the content is trivial.
#[tauri::command]
pub async fn suggest_timer_duration(
    content: String,
    storage: State<'_, Storage>,
) -> Result<u32, String> {
    let api_key = storage
        .get_setting("anthropic_api_key")
        .map_err(|e| e.to_string())?
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "No Anthropic API key configured. Set one in Settings → AI.".to_string())?;
    if content.trim().len() < 8 {
        // Too thin to suggest meaningfully; default to 25min pomodoro.
        return Ok(25 * 60);
    }
    let prompt = format!(
        "A user is about to start a focus timer for the task below. Suggest a duration in MINUTES. Reply with ONLY a single integer between 5 and 120. No words, no units, no punctuation.\n\nTask:\n{}",
        content.chars().take(500).collect::<String>()
    );
    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 10,
        "messages": [{ "role": "user", "content": prompt }],
    });
    let resp: serde_json::Value = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if let Some(err) = resp
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
    {
        return Err(err.to_string());
    }
    let text = resp
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| {
            arr.iter()
                .find_map(|p| p.get("text").and_then(|t| t.as_str()))
        })
        .unwrap_or("25")
        .trim()
        .to_string();
    let minutes: u32 = text
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or(25);
    let clamped = minutes.clamp(5, 120);
    Ok(clamped * 60)
}

/// Apply the sidebar layout to the controller window: a thin column
/// docked against the right edge of the primary monitor, full
/// height. Reverts to a normal floating window when `enabled` is
/// false.
#[tauri::command]
pub fn set_sidebar_mode(enabled: bool, app: AppHandle) -> Result<(), String> {
    let win = app
        .get_webview_window("controller")
        .ok_or_else(|| "controller window missing".to_string())?;
    if enabled {
        let monitor = win
            .current_monitor()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "no monitor".to_string())?;
        let width: i32 = 260;
        let size = monitor.size();
        let pos = monitor.position();
        win.set_size(tauri::PhysicalSize::new(width as u32, size.height))
            .map_err(|e| e.to_string())?;
        win.set_position(tauri::PhysicalPosition::new(
            pos.x + size.width as i32 - width,
            pos.y,
        ))
        .map_err(|e| e.to_string())?;
    } else {
        win.set_size(tauri::LogicalSize::new(360.0, 480.0))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn current_db_path(app: AppHandle) -> Result<String, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let pointer = data_dir.join("db_location.txt");
    let path = std::fs::read_to_string(&pointer)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| data_dir.join("postik.db").to_string_lossy().to_string());
    Ok(path)
}

/// Persist a new DB path. The new path takes effect on next launch —
/// re-opening SQLite mid-process would orphan every running query.
/// Pass `None` to clear the override and revert to the default
/// app_data_dir/postik.db.
#[tauri::command]
pub fn set_db_path(path: Option<String>, app: AppHandle) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    let pointer = data_dir.join("db_location.txt");
    match path {
        Some(p) if !p.trim().is_empty() => {
            std::fs::write(&pointer, p.trim()).map_err(|e| e.to_string())?;
        }
        _ => {
            let _ = std::fs::remove_file(&pointer);
        }
    }
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
