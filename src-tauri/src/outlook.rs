//! Outlook Calendar via Microsoft Graph. Mirrors `google.rs`'s OAuth
//! flow (PKCE, loopback redirect) but talks to login.microsoftonline
//! and graph.microsoft.com.
//!
//! Build-time env vars (set the same way as the Google ones):
//!
//! - `POSTIK_OUTLOOK_CLIENT_ID` — Application (client) ID from the
//!   Azure Portal app registration. PKCE-only public client, no secret
//!   required (and recommended NOT to ship one).

use crate::google::GoogleError;
use crate::storage::{GoogleAccount, GoogleEventRecord, Storage};
use crate::timer::{PostAction, TimerEngine, TimerMode};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;

const CLIENT_ID: Option<&str> = option_env!("POSTIK_OUTLOOK_CLIENT_ID");

const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const ME_URL: &str = "https://graph.microsoft.com/v1.0/me";
const EVENTS_URL: &str = "https://graph.microsoft.com/v1.0/me/calendarview";
const SCOPES: &str = "openid offline_access User.Read Calendars.Read";

pub fn is_configured() -> bool {
    matches!(CLIENT_ID, Some(id) if !id.is_empty())
}

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

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    #[serde(rename = "userPrincipalName")]
    user_principal_name: Option<String>,
    mail: Option<String>,
}

pub async fn connect(storage: &Storage) -> Result<crate::google::GoogleAccountInfo, GoogleError> {
    let client_id = CLIENT_ID
        .filter(|s| !s.is_empty())
        .ok_or(GoogleError::NotConfigured)?;
    let pkce = generate_pkce();

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
            ("response_mode", "query"),
            ("scope", SCOPES),
            ("code_challenge", pkce.challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("prompt", "select_account"),
        ],
    )
    .map_err(|e| GoogleError::OAuth(format!("auth url: {}", e)))?;
    crate::launcher::launch(None, Some(auth_url.as_str()));

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
                    let body = if code.is_some() {
                        "<html><body style=\"font-family:system-ui;padding:24px\">\
                         <h2>Postik connected to Outlook ✓</h2>\
                         <p>You can close this tab and return to Postik.</p>\
                         </body></html>"
                    } else {
                        "<html><body style=\"font-family:system-ui;padding:24px\">\
                         <h2>Connection failed</h2></body></html>"
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
                    return Err(GoogleError::OAuth("no code in redirect".into()));
                }
                Ok(None) => continue,
                Err(_) => return Err(GoogleError::OAuth("listener interrupted".into())),
            }
        }
    })
    .await
    .map_err(|e| GoogleError::OAuth(format!("listener task: {}", e)))??;

    let client = reqwest::Client::new();
    let token: TokenResponse = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("code", received_code.as_str()),
            ("code_verifier", pkce.verifier.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
            ("scope", SCOPES),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let refresh_token = token
        .refresh_token
        .clone()
        .ok_or_else(|| GoogleError::OAuth("Microsoft returned no refresh_token".into()))?;

    let user: UserInfo = client
        .get(ME_URL)
        .bearer_auth(&token.access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let now = chrono::Utc::now().timestamp();
    let account = GoogleAccount {
        email: user
            .mail
            .or(user.user_principal_name)
            .unwrap_or_else(|| "unknown@outlook".into()),
        access_token: token.access_token,
        refresh_token,
        access_token_expires_at: now + token.expires_in,
        connected_at: now,
    };
    storage.save_outlook_account(&account)?;
    Ok(account.into())
}

pub fn disconnect(storage: &Storage) -> Result<(), GoogleError> {
    storage.clear_outlook_account()?;
    Ok(())
}

pub fn account(storage: &Storage) -> Result<Option<crate::google::GoogleAccountInfo>, GoogleError> {
    Ok(storage.get_outlook_account()?.map(Into::into))
}

async fn ensure_fresh_token(storage: &Storage) -> Result<GoogleAccount, GoogleError> {
    let acc = storage
        .get_outlook_account()?
        .ok_or(GoogleError::NotConnected)?;
    let now = chrono::Utc::now().timestamp();
    if acc.access_token_expires_at > now + 30 {
        return Ok(acc);
    }
    let client_id = CLIENT_ID
        .filter(|s| !s.is_empty())
        .ok_or(GoogleError::NotConfigured)?;
    let token: TokenResponse = reqwest::Client::new()
        .post(TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("refresh_token", acc.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
            ("scope", SCOPES),
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
    storage.save_outlook_account(&refreshed)?;
    Ok(refreshed)
}

#[derive(Debug, Deserialize)]
struct EventList {
    value: Vec<Event>,
}

#[derive(Debug, Deserialize)]
struct Event {
    id: String,
    subject: Option<String>,
    body: Option<EventBody>,
    start: EventTime,
    end: EventTime,
    #[serde(rename = "webLink")]
    web_link: Option<String>,
    #[serde(default)]
    #[serde(rename = "isCancelled")]
    is_cancelled: bool,
}

#[derive(Debug, Deserialize)]
struct EventBody {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EventTime {
    #[serde(rename = "dateTime")]
    date_time: String,
}

impl EventTime {
    fn to_timestamp(&self) -> Option<i64> {
        // Outlook returns "2026-05-02T14:00:00.0000000" (no offset),
        // with the timezone reported separately. We treat it as UTC for
        // simplicity since Microsoft Graph's default is UTC anyway.
        let s = self.date_time.trim_end_matches('Z');
        let s = if s.contains('.') {
            s.split('.').next().unwrap_or(s)
        } else {
            s
        };
        let s = format!("{}Z", s);
        chrono::DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|d| d.timestamp())
    }
}

pub async fn sync(
    storage: &Storage,
    engine: &TimerEngine,
    range: crate::google::SyncRange,
) -> Result<Vec<GoogleEventRecord>, GoogleError> {
    let acc = ensure_fresh_token(storage).await?;
    let (start, end) = range.window();
    let time_min = chrono::DateTime::from_timestamp(start, 0)
        .unwrap_or_default()
        .to_rfc3339();
    let time_max = chrono::DateTime::from_timestamp(end, 0)
        .unwrap_or_default()
        .to_rfc3339();

    let resp: EventList = reqwest::Client::new()
        .get(EVENTS_URL)
        .bearer_auth(&acc.access_token)
        .query(&[
            ("startDateTime", time_min.as_str()),
            ("endDateTime", time_max.as_str()),
            ("$top", "100"),
            ("$orderby", "start/dateTime"),
        ])
        .header("Prefer", "outlook.timezone=\"UTC\"")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let now = chrono::Utc::now().timestamp();
    let mut kept: Vec<String> = Vec::with_capacity(resp.value.len());
    let mut anchor_x = 200.0_f64;
    let mut anchor_y = 200.0_f64;

    for ev in resp.value {
        if ev.is_cancelled {
            continue;
        }
        let (Some(start_ts), Some(end_ts)) = (ev.start.to_timestamp(), ev.end.to_timestamp())
        else {
            continue;
        };

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

        let (timer_armed, timer_offset_seconds) = existing
            .as_ref()
            .map(|e| (e.timer_armed, e.timer_offset_seconds))
            .unwrap_or((true, 0));

        let description = ev
            .body
            .and_then(|b| b.content)
            .map(|c| html_to_plain(&c))
            .unwrap_or_default();

        let record = GoogleEventRecord {
            event_id: ev.id.clone(),
            note_id: note_id.clone(),
            title: ev.subject.unwrap_or_else(|| "(no title)".into()),
            description,
            start_time: start_ts,
            end_time: end_ts,
            html_link: ev.web_link,
            timer_armed,
            timer_offset_seconds,
            synced_at: now,
        };
        storage.upsert_outlook_event(&record)?;
        schedule_event_timer(engine, &record, now);
        kept.push(ev.id);
    }

    storage.prune_outlook_events_outside(&kept)?;
    storage.list_outlook_events().map_err(GoogleError::from)
}

/// Outlook event bodies arrive as HTML; the existing note window
/// renders content as plain text (or markdown in preview mode), so a
/// minimal tag-strip + entity-decode keeps the output readable.
fn html_to_plain(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_string()
}

fn schedule_event_timer(engine: &TimerEngine, ev: &GoogleEventRecord, now: i64) {
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
        None,
    );
}
