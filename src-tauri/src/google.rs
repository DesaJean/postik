//! Google Calendar integration: OAuth (PKCE loopback), token refresh,
//! event sync. Designed to fail closed when no client credentials are
//! baked in at build time so dev builds work without setup.
//!
//! Build-time env vars (set in `src-tauri/.cargo/config.toml`'s `[env]`
//! block, or via the shell when invoking `cargo build`):
//!
//! - `POSTIK_GOOGLE_CLIENT_ID`     — required to enable the feature
//! - `POSTIK_GOOGLE_CLIENT_SECRET` — required (Google still issues one
//!   for "Desktop app" type credentials; PKCE provides the actual auth)

use crate::storage::{GoogleAccount, GoogleEventRecord, Storage};
use crate::timer::{PostAction, TimerEngine, TimerMode};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::TimeZone as _;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;

/// `option_env!` lets us ship a build that compiles without the secret
/// configured. The runtime checks return an empty option when the
/// developer hasn't set things up; the UI renders a setup-instruction
/// state instead of hitting the OAuth endpoint with an empty client_id.
const CLIENT_ID: Option<&str> = option_env!("POSTIK_GOOGLE_CLIENT_ID");
const CLIENT_SECRET: Option<&str> = option_env!("POSTIK_GOOGLE_CLIENT_SECRET");

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const CALENDAR_EVENTS_URL: &str = "https://www.googleapis.com/calendar/v3/calendars/primary/events";
const SCOPES: &str = "https://www.googleapis.com/auth/calendar.readonly \
                      https://www.googleapis.com/auth/userinfo.email";

#[derive(Debug, thiserror::Error)]
pub enum GoogleError {
    #[error("Google Calendar credentials are not configured for this build. See docs/google-calendar-setup.md.")]
    NotConfigured,
    #[error("not connected — connect a Google account first")]
    NotConnected,
    #[error("oauth: {0}")]
    OAuth(String),
    #[error("http: {0}")]
    Http(String),
    #[error("storage: {0}")]
    Storage(#[from] crate::storage::StorageError),
}

impl From<reqwest::Error> for GoogleError {
    fn from(e: reqwest::Error) -> Self {
        GoogleError::Http(e.to_string())
    }
}

pub fn is_configured() -> bool {
    matches!(
        (CLIENT_ID, CLIENT_SECRET),
        (Some(id), Some(secret)) if !id.is_empty() && !secret.is_empty()
    )
}

#[derive(Debug, Clone, Serialize)]
pub struct GoogleAccountInfo {
    pub email: String,
    pub connected_at: i64,
}

impl From<GoogleAccount> for GoogleAccountInfo {
    fn from(a: GoogleAccount) -> Self {
        Self {
            email: a.email,
            connected_at: a.connected_at,
        }
    }
}

// ---------------- PKCE ----------------

#[derive(Debug, Clone)]
struct Pkce {
    verifier: String,
    challenge: String,
}

fn generate_pkce() -> Pkce {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());
    Pkce {
        verifier,
        challenge,
    }
}

// ---------------- OAuth flow ----------------

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: Option<String>,
    #[allow(dead_code)]
    token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    email: String,
}

/// Run the full OAuth flow: spin up a loopback listener, open the user's
/// browser to Google's consent page, capture the redirect, exchange for
/// tokens, and persist the account. Blocks the calling task for as long
/// as it takes the user to grant consent — cap at 5 minutes.
pub async fn connect(storage: &Storage) -> Result<GoogleAccountInfo, GoogleError> {
    let client_id = CLIENT_ID
        .filter(|s| !s.is_empty())
        .ok_or(GoogleError::NotConfigured)?;
    let client_secret = CLIENT_SECRET
        .filter(|s| !s.is_empty())
        .ok_or(GoogleError::NotConfigured)?;

    let pkce = generate_pkce();

    // Bind to a random ephemeral port so we don't collide with anything
    // the user is running. tiny_http binds synchronously; do that on a
    // blocking thread and hand the listener back via a channel.
    let server = tiny_http::Server::http("127.0.0.1:0")
        .map_err(|e| GoogleError::OAuth(format!("loopback listener: {}", e)))?;
    let port = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| GoogleError::OAuth("listener has no port".into()))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let auth_url = url::Url::parse_with_params(
        AUTH_URL,
        &[
            ("client_id", client_id),
            ("redirect_uri", redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", SCOPES),
            ("access_type", "offline"),
            ("prompt", "consent"),
            ("code_challenge", pkce.challenge.as_str()),
            ("code_challenge_method", "S256"),
        ],
    )
    .map_err(|e| GoogleError::OAuth(format!("auth url: {}", e)))?;
    crate::launcher::launch(None, Some(auth_url.as_str()));

    // Wait for the redirect on a blocking task, with a 5-minute budget.
    let received_code = tokio::task::spawn_blocking(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(300);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return Err(GoogleError::OAuth("timed out waiting for consent".into()));
            }
            match server.recv_timeout(remaining) {
                Ok(Some(req)) => {
                    let url_str = format!("http://127.0.0.1:{}{}", port, req.url());
                    let parsed = url::Url::parse(&url_str)
                        .map_err(|e| GoogleError::OAuth(format!("redirect url: {}", e)))?;
                    let code = parsed
                        .query_pairs()
                        .find(|(k, _)| k == "code")
                        .map(|(_, v)| v.into_owned());
                    let err = parsed
                        .query_pairs()
                        .find(|(k, _)| k == "error")
                        .map(|(_, v)| v.into_owned());
                    let body = if code.is_some() {
                        "<html><body style=\"font-family:system-ui;padding:24px\">\
                         <h2>Postik connected ✓</h2>\
                         <p>You can close this tab and return to Postik.</p>\
                         </body></html>"
                    } else {
                        "<html><body style=\"font-family:system-ui;padding:24px\">\
                         <h2>Postik connection failed</h2>\
                         <p>Return to Postik and try again.</p>\
                         </body></html>"
                    };
                    let response = tiny_http::Response::from_string(body).with_header(
                        "Content-Type: text/html; charset=utf-8"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    );
                    let _ = req.respond(response);
                    if let Some(c) = code {
                        return Ok(c);
                    }
                    return Err(GoogleError::OAuth(
                        err.unwrap_or_else(|| "no code in redirect".into()),
                    ));
                }
                Ok(None) => continue,
                Err(_) => {
                    return Err(GoogleError::OAuth("listener interrupted".into()));
                }
            }
        }
    })
    .await
    .map_err(|e| GoogleError::OAuth(format!("listener task: {}", e)))??;

    // Exchange the code for tokens.
    let client = reqwest::Client::new();
    let token: TokenResponse = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", received_code.as_str()),
            ("code_verifier", pkce.verifier.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let refresh_token = token
        .refresh_token
        .clone()
        .ok_or_else(|| GoogleError::OAuth("Google returned no refresh_token".into()))?;

    // Identify the connected account.
    let user: UserInfo = client
        .get(USERINFO_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let now = chrono::Utc::now().timestamp();
    let account = GoogleAccount {
        email: user.email,
        access_token: token.access_token,
        refresh_token,
        access_token_expires_at: now + token.expires_in,
        connected_at: now,
    };
    storage.save_google_account(&account)?;
    Ok(account.into())
}

pub fn disconnect(storage: &Storage) -> Result<(), GoogleError> {
    storage.clear_google_account()?;
    Ok(())
}

pub fn account(storage: &Storage) -> Result<Option<GoogleAccountInfo>, GoogleError> {
    Ok(storage.get_google_account()?.map(Into::into))
}

/// Refresh the access token if expired. Returns the (possibly-updated)
/// account snapshot. The refresh_token is reused; only the access_token
/// + expiry change.
async fn ensure_fresh_token(storage: &Storage) -> Result<GoogleAccount, GoogleError> {
    let acc = storage
        .get_google_account()?
        .ok_or(GoogleError::NotConnected)?;
    let now = chrono::Utc::now().timestamp();
    // Refresh slightly before expiry to avoid edge-case 401s.
    if acc.access_token_expires_at > now + 30 {
        return Ok(acc);
    }
    let client_id = CLIENT_ID
        .filter(|s| !s.is_empty())
        .ok_or(GoogleError::NotConfigured)?;
    let client_secret = CLIENT_SECRET
        .filter(|s| !s.is_empty())
        .ok_or(GoogleError::NotConfigured)?;
    let token: TokenResponse = reqwest::Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", acc.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let refreshed = GoogleAccount {
        access_token: token.access_token,
        access_token_expires_at: now + token.expires_in,
        refresh_token: token.refresh_token.unwrap_or(acc.refresh_token),
        ..acc
    };
    storage.save_google_account(&refreshed)?;
    Ok(refreshed)
}

// ---------------- Calendar API ----------------

#[derive(Debug, Deserialize)]
struct CalendarEventList {
    items: Vec<CalendarEvent>,
}

#[derive(Debug, Deserialize)]
struct CalendarEvent {
    id: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    description: Option<String>,
    start: CalendarEventTime,
    end: CalendarEventTime,
    #[serde(rename = "htmlLink", default)]
    html_link: Option<String>,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CalendarEventTime {
    #[serde(rename = "dateTime", default)]
    date_time: Option<String>,
    #[serde(default)]
    date: Option<String>,
}

impl CalendarEventTime {
    /// Resolve to a unix timestamp. All-day events (with `date` only)
    /// anchor at 09:00 local time so they show up alongside other
    /// morning events; this is documented as a v1 limitation.
    fn to_timestamp(&self) -> Option<i64> {
        if let Some(dt) = &self.date_time {
            return chrono::DateTime::parse_from_rfc3339(dt)
                .ok()
                .map(|d| d.timestamp());
        }
        if let Some(d) = &self.date {
            // YYYY-MM-DD interpreted at 09:00 local time.
            let date = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()?;
            let dt = date.and_hms_opt(9, 0, 0)?;
            return chrono::Local
                .from_local_datetime(&dt)
                .single()
                .map(|x| x.timestamp());
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SyncRange {
    Today,
    SevenDays,
    Custom { start: i64, end: i64 },
}

impl SyncRange {
    pub fn from_kind(kind: &str, start: Option<i64>, end: Option<i64>) -> Option<Self> {
        match kind {
            "today" => Some(SyncRange::Today),
            "seven_days" | "7days" => Some(SyncRange::SevenDays),
            "custom" => {
                let s = start?;
                let e = end?;
                Some(SyncRange::Custom { start: s, end: e })
            }
            _ => None,
        }
    }

    fn to_window(self) -> (i64, i64) {
        match self {
            SyncRange::Today => {
                let now = chrono::Local::now();
                let start_of_today = now
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .and_then(|dt| chrono::Local.from_local_datetime(&dt).single())
                    .map(|x| x.timestamp())
                    .unwrap_or_else(|| now.timestamp());
                (start_of_today, start_of_today + 24 * 3600)
            }
            SyncRange::SevenDays => {
                let now = chrono::Local::now().timestamp();
                (now, now + 7 * 24 * 3600)
            }
            SyncRange::Custom { start, end } => (start, end),
        }
    }
}

/// Run a full sync: fetch events in `range`, upsert into storage,
/// reschedule per-event timers via the timer engine, and prune
/// dropped events. Returns the up-to-date list (sorted by start_time).
pub async fn sync(
    storage: &Storage,
    engine: &TimerEngine,
    range: SyncRange,
) -> Result<Vec<GoogleEventRecord>, GoogleError> {
    let acc = ensure_fresh_token(storage).await?;
    let (start, end) = range.to_window();
    let time_min = chrono::DateTime::from_timestamp(start, 0)
        .unwrap_or_default()
        .to_rfc3339();
    let time_max = chrono::DateTime::from_timestamp(end, 0)
        .unwrap_or_default()
        .to_rfc3339();

    let resp: CalendarEventList = reqwest::Client::new()
        .get(CALENDAR_EVENTS_URL)
        .bearer_auth(&acc.access_token)
        .query(&[
            ("timeMin", time_min.as_str()),
            ("timeMax", time_max.as_str()),
            ("singleEvents", "true"),
            ("orderBy", "startTime"),
            ("maxResults", "100"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let now = chrono::Utc::now().timestamp();
    let mut kept_event_ids = Vec::with_capacity(resp.items.len());
    let mut anchor_x = 140.0_f64;
    let mut anchor_y = 140.0_f64;

    for ev in resp.items {
        if ev.status.as_deref() == Some("cancelled") {
            continue;
        }
        let (Some(start_ts), Some(end_ts)) = (ev.start.to_timestamp(), ev.end.to_timestamp())
        else {
            continue;
        };

        // Find or create the linked note.
        let existing = storage.get_google_event(&ev.id)?;
        let note_id = match existing.as_ref() {
            Some(e) => e.note_id.clone(),
            None => {
                let n = storage.create_event_note(&ev.id, anchor_x, anchor_y)?;
                anchor_x += 32.0;
                anchor_y += 32.0;
                n.id
            }
        };

        // Carry over user-controlled timer fields when updating; default
        // them on first insert.
        let (timer_armed, timer_offset_seconds) = existing
            .as_ref()
            .map(|e| (e.timer_armed, e.timer_offset_seconds))
            .unwrap_or((true, 0));

        let record = GoogleEventRecord {
            event_id: ev.id.clone(),
            note_id: note_id.clone(),
            title: ev.summary.unwrap_or_else(|| "(no title)".into()),
            description: ev.description.unwrap_or_default(),
            start_time: start_ts,
            end_time: end_ts,
            html_link: ev.html_link,
            timer_armed,
            timer_offset_seconds,
            synced_at: now,
        };
        storage.upsert_google_event(&record)?;
        schedule_event_timer(engine, &record, now);
        kept_event_ids.push(ev.id);
    }

    storage.prune_events_outside(&kept_event_ids)?;
    storage.list_google_events().map_err(GoogleError::from)
}

/// (Re)schedule the countdown for an event. Cancels any existing timer
/// on the same note first. No-op if the offset would put the fire time
/// in the past, or if `timer_armed` is false.
pub fn schedule_event_timer(engine: &TimerEngine, ev: &GoogleEventRecord, now: i64) {
    engine.cancel(&ev.note_id);
    if !ev.timer_armed {
        return;
    }
    let fire_at = ev.start_time - ev.timer_offset_seconds;
    let duration = fire_at - now;
    if duration <= 0 {
        return;
    }
    engine.start(
        ev.note_id.clone(),
        TimerMode::Countdown,
        Some(duration),
        None,
        PostAction::default(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_is_url_safe_base64_of_sha256() {
        let p = generate_pkce();
        assert!(!p.verifier.is_empty());
        assert!(!p.challenge.is_empty());
        // Recompute and verify.
        let mut hasher = Sha256::new();
        hasher.update(p.verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(hasher.finalize());
        assert_eq!(p.challenge, expected);
    }

    #[test]
    fn sync_range_kinds_parse() {
        assert!(matches!(
            SyncRange::from_kind("today", None, None),
            Some(SyncRange::Today)
        ));
        assert!(matches!(
            SyncRange::from_kind("seven_days", None, None),
            Some(SyncRange::SevenDays)
        ));
        assert!(matches!(
            SyncRange::from_kind("custom", Some(1), Some(2)),
            Some(SyncRange::Custom { start: 1, end: 2 })
        ));
        assert!(SyncRange::from_kind("custom", None, Some(2)).is_none());
        assert!(SyncRange::from_kind("garbage", None, None).is_none());
    }

    #[test]
    fn calendar_event_time_resolves_datetime() {
        let t = CalendarEventTime {
            date_time: Some("2026-05-01T14:30:00-03:00".into()),
            date: None,
        };
        assert!(t.to_timestamp().is_some());
    }
}
