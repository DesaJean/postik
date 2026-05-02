//! Recurring per-note schedules. Each rule is stored on the note row
//! as a JSON object: `{"hour":9,"minute":0,"days":[1,2,3,4,5]}`. Days
//! follow JS `getDay()` semantics (0=Sun..6=Sat) so the frontend doesn't
//! have to translate.
//!
//! A tokio task wakes once a minute, evaluates every rule, and fires
//! the matching notes. "Firing" means: focus the note window AND show
//! a system notification. We also stamp `recurring_last_fired` with the
//! current minute key so a hot-loop wake-up doesn't re-fire within the
//! same minute.

use crate::storage::{NoteRecord, Storage};
use crate::window_manager::WindowManager;
use chrono::{Datelike, Local, Timelike};
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Deserialize)]
struct Rule {
    hour: u32,
    minute: u32,
    /// JS getDay() — 0=Sun, 1=Mon, ..., 6=Sat.
    days: Vec<u32>,
}

fn parse_rule(json: &str) -> Option<Rule> {
    serde_json::from_str(json).ok()
}

/// Returns Some(minute_key) when the rule should fire at `now`, where
/// the minute_key is a deduplication token (`YYYY-MM-DDTHH:MM`).
fn matches(rule: &Rule, now: chrono::DateTime<Local>) -> Option<String> {
    if rule.hour != now.hour() {
        return None;
    }
    if rule.minute != now.minute() {
        return None;
    }
    let weekday_js: u32 = now.weekday().num_days_from_sunday();
    if !rule.days.contains(&weekday_js) {
        return None;
    }
    Some(now.format("%Y-%m-%dT%H:%M").to_string())
}

/// One scheduler tick. Inspects every recurring rule and fires the
/// matching notes. Returns the count of notes that fired (mainly useful
/// for tests and logs).
pub fn tick(app: &AppHandle, storage: &Storage, wm: &WindowManager) -> usize {
    let notes = match storage.list_recurring_notes() {
        Ok(n) => n,
        Err(e) => {
            log::warn!("recurring: list_recurring_notes failed: {e}");
            return 0;
        }
    };
    let now = Local::now();
    let mut fired = 0;
    for note in notes {
        let Some(rule_json) = note.recurring_rule.as_deref() else {
            continue;
        };
        let Some(rule) = parse_rule(rule_json) else {
            continue;
        };
        let Some(minute_key) = matches(&rule, now) else {
            continue;
        };
        if note.recurring_last_fired.as_deref() == Some(&minute_key) {
            continue;
        }
        fire_note(app, wm, &note);
        if let Err(e) = storage.mark_recurring_fired(&note.id, &minute_key) {
            log::warn!("recurring: mark_recurring_fired failed: {e}");
        }
        fired += 1;
    }
    fired
}

fn fire_note(app: &AppHandle, wm: &WindowManager, note: &NoteRecord) {
    // Focus (and create) the window so the user actually sees the note.
    if let Err(e) = wm.focus_note(app, &note.id) {
        log::warn!("recurring: focus_note({}) failed: {e}", note.id);
    }
    // Native notification with the first line of the content as body.
    let title = "⏰ Recurring reminder".to_string();
    let body = note
        .content
        .lines()
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("(empty note)")
        .chars()
        .take(80)
        .collect::<String>();
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        log::warn!("recurring: notification show failed: {e}");
    }
}

/// Spawn the scheduler task. Wakes once a minute and evaluates rules.
/// Aligned to the start of each minute for predictable fire times.
pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // Sleep until the next minute boundary, plus a 1s slack.
            let now = Local::now();
            let secs_into_minute = now.second() as u64;
            let sleep_for = 60 - secs_into_minute + 1;
            tokio::time::sleep(std::time::Duration::from_secs(sleep_for)).await;
            let storage: tauri::State<Storage> = app.state();
            let wm: tauri::State<WindowManager> = app.state();
            let _fired = tick(&app, &storage, &wm);
        }
    });
}
