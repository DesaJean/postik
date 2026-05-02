use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

const DEFAULT_COLOR: &str = "amber";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRecord {
    pub id: String,
    pub content: String,
    pub color_id: String,
    pub opacity: f64,
    pub always_on_top: bool,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub created_at: i64,
    pub updated_at: i64,
    /// Optional text-color override. `None` (or "auto") means inherit from
    /// the palette's default text color. Otherwise a known token like "dark",
    /// "medium", "light", or "accent".
    #[serde(default)]
    pub text_color: Option<String>,
    /// When set, this note is backed by a Google Calendar event. The
    /// content (title/description) and timer are synced from Google and
    /// the note window renders in read-only mode. Editable notes have
    /// `event_id = None`.
    #[serde(default)]
    pub event_id: Option<String>,
    /// Comma-separated lowercase tag list. `None` and empty string both
    /// mean untagged. The frontend splits/joins; storage stays flat.
    #[serde(default)]
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAccount {
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: i64,
    pub connected_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleEventRecord {
    pub event_id: String,
    pub note_id: String,
    pub title: String,
    pub description: String,
    pub start_time: i64,
    pub end_time: i64,
    pub html_link: Option<String>,
    pub timer_armed: bool,
    pub timer_offset_seconds: i64,
    pub synced_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerRecord {
    pub note_id: String,
    pub mode: String,
    pub duration_seconds: Option<i64>,
    pub elapsed_seconds: i64,
    pub state: String,
    pub pomodoro_phase: Option<String>,
    pub started_at: Option<i64>,
    /// Path to the executable / .app bundle to launch when the timer ends.
    /// `None` means no post-timer action.
    #[serde(default)]
    pub post_action_path: Option<String>,
    /// Optional argument string (typically a URL) passed to the launched
    /// app, or — when `post_action_path` is `None` and this is a URL —
    /// opened in the default browser.
    #[serde(default)]
    pub post_action_args: Option<String>,
    /// For pomodoro: total work cycles before auto-ending. `None` keeps the
    /// legacy infinite cycling behaviour.
    #[serde(default)]
    pub pomodoro_total_cycles: Option<i64>,
    /// For pomodoro: how many work cycles have completed so far.
    #[serde(default)]
    pub pomodoro_completed_cycles: Option<i64>,
}

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Self::init(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open an in-memory database, used by unit tests.
    #[cfg(test)]
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Self::init(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL DEFAULT '',
                color_id TEXT NOT NULL DEFAULT 'amber',
                opacity REAL NOT NULL DEFAULT 1.0,
                always_on_top INTEGER NOT NULL DEFAULT 1,
                x REAL NOT NULL,
                y REAL NOT NULL,
                width REAL NOT NULL,
                height REAL NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS timers (
                note_id TEXT PRIMARY KEY,
                mode TEXT NOT NULL,
                duration_seconds INTEGER,
                elapsed_seconds INTEGER NOT NULL DEFAULT 0,
                state TEXT NOT NULL DEFAULT 'idle',
                pomodoro_phase TEXT,
                started_at INTEGER,
                FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS google_account (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                email TEXT NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                access_token_expires_at INTEGER NOT NULL,
                connected_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS google_events (
                event_id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                start_time INTEGER NOT NULL,
                end_time INTEGER NOT NULL,
                html_link TEXT,
                timer_armed INTEGER NOT NULL DEFAULT 1,
                timer_offset_seconds INTEGER NOT NULL DEFAULT 0,
                synced_at INTEGER NOT NULL,
                FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE
            );
            ",
        )?;
        // Add `text_color` column for older DBs that pre-date the feature.
        // SQLite has no `IF NOT EXISTS` for ADD COLUMN; ignoring the
        // duplicate-column error is the standard idiom.
        let _ = conn.execute("ALTER TABLE notes ADD COLUMN text_color TEXT", []);
        // Post-timer action + pomodoro cycle accounting. Same idiom.
        let _ = conn.execute("ALTER TABLE timers ADD COLUMN post_action_path TEXT", []);
        let _ = conn.execute("ALTER TABLE timers ADD COLUMN post_action_args TEXT", []);
        let _ = conn.execute(
            "ALTER TABLE timers ADD COLUMN pomodoro_total_cycles INTEGER",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE timers ADD COLUMN pomodoro_completed_cycles INTEGER",
            [],
        );
        // Notes get a nullable `event_id` so a row can be marked as backed
        // by a Google Calendar event (read-only in the UI).
        let _ = conn.execute("ALTER TABLE notes ADD COLUMN event_id TEXT", []);
        // Manual sort order. NULL means "fall back to updated_at" so the
        // existing default-sort behaviour is unchanged for users who never
        // reorder. After a reorder, every visible row gets an explicit
        // index.
        let _ = conn.execute("ALTER TABLE notes ADD COLUMN sort_index INTEGER", []);
        // Soft-delete: NULL = active, non-NULL = archived-at timestamp.
        // The list_notes query filters archived rows out by default; the
        // archive view uses list_archived_notes.
        let _ = conn.execute("ALTER TABLE notes ADD COLUMN archived_at INTEGER", []);
        // Tags as a comma-separated lowercase string (`work,1on1,urgent`).
        // Empty string and NULL are equivalent — both mean "untagged".
        // Frontend handles parsing/joining; storage stays flat.
        let _ = conn.execute("ALTER TABLE notes ADD COLUMN tags TEXT", []);
        Ok(())
    }

    pub fn create_note(&self, x: f64, y: f64, width: f64, height: f64) -> Result<NoteRecord> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO notes (id, content, color_id, opacity, always_on_top, x, y, width, height, created_at, updated_at)
             VALUES (?1, '', ?2, 1.0, 1, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![id, DEFAULT_COLOR, x, y, width, height, now],
        )?;
        Ok(NoteRecord {
            id,
            content: String::new(),
            color_id: DEFAULT_COLOR.to_string(),
            opacity: 1.0,
            always_on_top: true,
            x,
            y,
            width,
            height,
            created_at: now,
            updated_at: now,
            text_color: None,
            event_id: None,
            tags: None,
        })
    }

    pub fn list_notes(&self) -> Result<Vec<NoteRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, content, color_id, opacity, always_on_top, x, y, width, height, created_at, updated_at, text_color, event_id, tags
             FROM notes
             WHERE archived_at IS NULL
             ORDER BY
               CASE WHEN sort_index IS NULL THEN 1 ELSE 0 END,
               sort_index ASC,
               updated_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(NoteRecord {
                id: r.get(0)?,
                content: r.get(1)?,
                color_id: r.get(2)?,
                opacity: r.get(3)?,
                always_on_top: r.get::<_, i64>(4)? != 0,
                x: r.get(5)?,
                y: r.get(6)?,
                width: r.get(7)?,
                height: r.get(8)?,
                created_at: r.get(9)?,
                updated_at: r.get(10)?,
                text_color: r.get(11)?,
                event_id: r.get(12)?,
                tags: r.get(13)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get_note(&self, id: &str) -> Result<Option<NoteRecord>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, content, color_id, opacity, always_on_top, x, y, width, height, created_at, updated_at, text_color, event_id, tags
             FROM notes WHERE id = ?1",
            [id],
            |r| {
                Ok(NoteRecord {
                    id: r.get(0)?,
                    content: r.get(1)?,
                    color_id: r.get(2)?,
                    opacity: r.get(3)?,
                    always_on_top: r.get::<_, i64>(4)? != 0,
                    x: r.get(5)?,
                    y: r.get(6)?,
                    width: r.get(7)?,
                    height: r.get(8)?,
                    created_at: r.get(9)?,
                    updated_at: r.get(10)?,
                    text_color: r.get(11)?,
                    event_id: r.get(12)?,
                    tags: r.get(13)?,
                })
            },
        )
        .optional()
        .map_err(StorageError::from)
    }

    pub fn update_content(&self, id: &str, content: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
            params![content, now, id],
        )?;
        Ok(())
    }

    pub fn update_color(&self, id: &str, color_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET color_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![color_id, now, id],
        )?;
        Ok(())
    }

    pub fn update_tags(&self, id: &str, tags: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET tags = ?1, updated_at = ?2 WHERE id = ?3",
            params![tags, now, id],
        )?;
        Ok(())
    }

    pub fn update_text_color(&self, id: &str, text_color: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET text_color = ?1, updated_at = ?2 WHERE id = ?3",
            params![text_color, now, id],
        )?;
        Ok(())
    }

    pub fn update_opacity(&self, id: &str, opacity: f64) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET opacity = ?1, updated_at = ?2 WHERE id = ?3",
            params![opacity, now, id],
        )?;
        Ok(())
    }

    pub fn update_position(&self, id: &str, x: f64, y: f64) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET x = ?1, y = ?2, updated_at = ?3 WHERE id = ?4",
            params![x, y, now, id],
        )?;
        Ok(())
    }

    pub fn update_size(&self, id: &str, width: f64, height: f64) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET width = ?1, height = ?2, updated_at = ?3 WHERE id = ?4",
            params![width, height, now, id],
        )?;
        Ok(())
    }

    pub fn update_always_on_top(&self, id: &str, value: bool) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET always_on_top = ?1, updated_at = ?2 WHERE id = ?3",
            params![value as i64, now, id],
        )?;
        Ok(())
    }

    pub fn delete_note(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn archive_note(&self, id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET archived_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn unarchive_note(&self, id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notes SET archived_at = NULL, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn list_archived_notes(&self) -> Result<Vec<NoteRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, content, color_id, opacity, always_on_top, x, y, width, height, created_at, updated_at, text_color, event_id, tags
             FROM notes
             WHERE archived_at IS NOT NULL
             ORDER BY archived_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(NoteRecord {
                id: r.get(0)?,
                content: r.get(1)?,
                color_id: r.get(2)?,
                opacity: r.get(3)?,
                always_on_top: r.get::<_, i64>(4)? != 0,
                x: r.get(5)?,
                y: r.get(6)?,
                width: r.get(7)?,
                height: r.get(8)?,
                created_at: r.get(9)?,
                updated_at: r.get(10)?,
                text_color: r.get(11)?,
                event_id: r.get(12)?,
                tags: r.get(13)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Assign sort_index 0..N to the notes in `ordered_ids` in array
    /// order. Notes not in the list keep their existing index (or NULL).
    /// All updates run in a single transaction.
    pub fn reorder_notes(&self, ordered_ids: &[String]) -> Result<()> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        for (idx, id) in ordered_ids.iter().enumerate() {
            tx.execute(
                "UPDATE notes SET sort_index = ?1 WHERE id = ?2",
                params![idx as i64, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_timer(&self, t: &TimerRecord) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO timers (
                note_id, mode, duration_seconds, elapsed_seconds, state,
                pomodoro_phase, started_at,
                post_action_path, post_action_args,
                pomodoro_total_cycles, pomodoro_completed_cycles
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(note_id) DO UPDATE SET
                mode = excluded.mode,
                duration_seconds = excluded.duration_seconds,
                elapsed_seconds = excluded.elapsed_seconds,
                state = excluded.state,
                pomodoro_phase = excluded.pomodoro_phase,
                started_at = excluded.started_at,
                post_action_path = excluded.post_action_path,
                post_action_args = excluded.post_action_args,
                pomodoro_total_cycles = excluded.pomodoro_total_cycles,
                pomodoro_completed_cycles = excluded.pomodoro_completed_cycles",
            params![
                t.note_id,
                t.mode,
                t.duration_seconds,
                t.elapsed_seconds,
                t.state,
                t.pomodoro_phase,
                t.started_at,
                t.post_action_path,
                t.post_action_args,
                t.pomodoro_total_cycles,
                t.pomodoro_completed_cycles,
            ],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_timer(&self, note_id: &str) -> Result<Option<TimerRecord>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT note_id, mode, duration_seconds, elapsed_seconds, state,
                    pomodoro_phase, started_at,
                    post_action_path, post_action_args,
                    pomodoro_total_cycles, pomodoro_completed_cycles
             FROM timers WHERE note_id = ?1",
            [note_id],
            |r| {
                Ok(TimerRecord {
                    note_id: r.get(0)?,
                    mode: r.get(1)?,
                    duration_seconds: r.get(2)?,
                    elapsed_seconds: r.get(3)?,
                    state: r.get(4)?,
                    pomodoro_phase: r.get(5)?,
                    started_at: r.get(6)?,
                    post_action_path: r.get(7)?,
                    post_action_args: r.get(8)?,
                    pomodoro_total_cycles: r.get(9)?,
                    pomodoro_completed_cycles: r.get(10)?,
                })
            },
        )
        .optional()
        .map_err(StorageError::from)
    }

    pub fn delete_timer(&self, note_id: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM timers WHERE note_id = ?1", [note_id])?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock();
        conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get::<_, String>(0)
        })
        .optional()
        .map_err(StorageError::from)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn list_settings(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    // ---------------- Google Calendar ----------------

    pub fn save_google_account(&self, acc: &GoogleAccount) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO google_account (id, email, access_token, refresh_token, access_token_expires_at, connected_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                email = excluded.email,
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                access_token_expires_at = excluded.access_token_expires_at,
                connected_at = excluded.connected_at",
            params![
                acc.email,
                acc.access_token,
                acc.refresh_token,
                acc.access_token_expires_at,
                acc.connected_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_google_account(&self) -> Result<Option<GoogleAccount>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT email, access_token, refresh_token, access_token_expires_at, connected_at
             FROM google_account WHERE id = 1",
            [],
            |r| {
                Ok(GoogleAccount {
                    email: r.get(0)?,
                    access_token: r.get(1)?,
                    refresh_token: r.get(2)?,
                    access_token_expires_at: r.get(3)?,
                    connected_at: r.get(4)?,
                })
            },
        )
        .optional()
        .map_err(StorageError::from)
    }

    pub fn clear_google_account(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM google_account WHERE id = 1", [])?;
        // Cascade delete the event-backed notes, which cascades through to
        // their google_events rows via the FK.
        conn.execute("DELETE FROM notes WHERE event_id IS NOT NULL", [])?;
        Ok(())
    }

    /// Insert a row into `notes` linked to a Google event. Used by the
    /// sync loop when an event arrives that has no local row yet. The
    /// position is randomised within a small offset so multiple events
    /// don't open stacked exactly on top of each other.
    pub fn create_event_note(&self, event_id: &str, x: f64, y: f64) -> Result<NoteRecord> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO notes (id, content, color_id, opacity, always_on_top, x, y, width, height, created_at, updated_at, event_id)
             VALUES (?1, '', 'blue', 1.0, 1, ?2, ?3, 280.0, 200.0, ?4, ?4, ?5)",
            params![id, x, y, now, event_id],
        )?;
        Ok(NoteRecord {
            id,
            content: String::new(),
            color_id: "blue".to_string(),
            opacity: 1.0,
            always_on_top: true,
            x,
            y,
            width: 280.0,
            height: 200.0,
            created_at: now,
            updated_at: now,
            text_color: None,
            event_id: Some(event_id.to_string()),
            tags: None,
        })
    }

    pub fn upsert_google_event(&self, e: &GoogleEventRecord) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO google_events (event_id, note_id, title, description, start_time, end_time, html_link, timer_armed, timer_offset_seconds, synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(event_id) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                start_time = excluded.start_time,
                end_time = excluded.end_time,
                html_link = excluded.html_link,
                synced_at = excluded.synced_at",
            params![
                e.event_id,
                e.note_id,
                e.title,
                e.description,
                e.start_time,
                e.end_time,
                e.html_link,
                e.timer_armed as i64,
                e.timer_offset_seconds,
                e.synced_at,
            ],
        )?;
        // Also write the description into the linked note's content so the
        // existing note window can render it without a special path.
        let preview = if e.description.is_empty() {
            e.title.clone()
        } else {
            format!("{}\n\n{}", e.title, e.description)
        };
        conn.execute(
            "UPDATE notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
            params![preview, e.synced_at, e.note_id],
        )?;
        Ok(())
    }

    pub fn list_google_events(&self) -> Result<Vec<GoogleEventRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT event_id, note_id, title, description, start_time, end_time, html_link, timer_armed, timer_offset_seconds, synced_at
             FROM google_events ORDER BY start_time ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(GoogleEventRecord {
                event_id: r.get(0)?,
                note_id: r.get(1)?,
                title: r.get(2)?,
                description: r.get(3)?,
                start_time: r.get(4)?,
                end_time: r.get(5)?,
                html_link: r.get(6)?,
                timer_armed: r.get::<_, i64>(7)? != 0,
                timer_offset_seconds: r.get(8)?,
                synced_at: r.get(9)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get_google_event(&self, event_id: &str) -> Result<Option<GoogleEventRecord>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT event_id, note_id, title, description, start_time, end_time, html_link, timer_armed, timer_offset_seconds, synced_at
             FROM google_events WHERE event_id = ?1",
            [event_id],
            |r| {
                Ok(GoogleEventRecord {
                    event_id: r.get(0)?,
                    note_id: r.get(1)?,
                    title: r.get(2)?,
                    description: r.get(3)?,
                    start_time: r.get(4)?,
                    end_time: r.get(5)?,
                    html_link: r.get(6)?,
                    timer_armed: r.get::<_, i64>(7)? != 0,
                    timer_offset_seconds: r.get(8)?,
                    synced_at: r.get(9)?,
                })
            },
        )
        .optional()
        .map_err(StorageError::from)
    }

    pub fn set_event_timer(&self, event_id: &str, armed: bool, offset_seconds: i64) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE google_events SET timer_armed = ?1, timer_offset_seconds = ?2 WHERE event_id = ?3",
            params![armed as i64, offset_seconds, event_id],
        )?;
        Ok(())
    }

    /// Delete every event whose id is NOT in `keep`. Used after a sync to
    /// prune events Google no longer returns. Their linked notes cascade.
    pub fn prune_events_outside(&self, keep: &[String]) -> Result<()> {
        let conn = self.conn.lock();
        if keep.is_empty() {
            // Wipe all event-notes (and their google_events via cascade).
            conn.execute("DELETE FROM notes WHERE event_id IS NOT NULL", [])?;
            return Ok(());
        }
        // Build a parameterised IN (?, ?, ?) query.
        let placeholders = std::iter::repeat("?")
            .take(keep.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "DELETE FROM notes WHERE event_id IS NOT NULL AND event_id NOT IN ({})",
            placeholders,
        );
        let params_vec: Vec<&dyn rusqlite::ToSql> =
            keep.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        conn.execute(&sql, params_vec.as_slice())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open() -> Storage {
        Storage::open_memory().expect("open memory db")
    }

    #[test]
    fn create_and_list_notes() {
        let s = open();
        let n = s.create_note(10.0, 20.0, 240.0, 200.0).unwrap();
        let all = s.list_notes().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, n.id);
        assert_eq!(all[0].color_id, "amber");
        assert!(all[0].always_on_top);
    }

    #[test]
    fn update_fields() {
        let s = open();
        let n = s.create_note(0.0, 0.0, 240.0, 200.0).unwrap();
        s.update_content(&n.id, "hello").unwrap();
        s.update_color(&n.id, "teal").unwrap();
        s.update_opacity(&n.id, 0.5).unwrap();
        s.update_always_on_top(&n.id, false).unwrap();

        let r = s.get_note(&n.id).unwrap().unwrap();
        assert_eq!(r.content, "hello");
        assert_eq!(r.color_id, "teal");
        assert!((r.opacity - 0.5).abs() < f64::EPSILON);
        assert!(!r.always_on_top);
    }

    #[test]
    fn delete_note_cascades_timer() {
        let s = open();
        let n = s.create_note(0.0, 0.0, 240.0, 200.0).unwrap();
        s.upsert_timer(&TimerRecord {
            note_id: n.id.clone(),
            mode: "countdown".into(),
            duration_seconds: Some(60),
            elapsed_seconds: 0,
            state: "running".into(),
            pomodoro_phase: None,
            started_at: Some(1),
            post_action_path: None,
            post_action_args: None,
            pomodoro_total_cycles: None,
            pomodoro_completed_cycles: None,
        })
        .unwrap();
        assert!(s.get_timer(&n.id).unwrap().is_some());
        s.delete_note(&n.id).unwrap();
        assert!(s.get_note(&n.id).unwrap().is_none());
        assert!(s.get_timer(&n.id).unwrap().is_none());
    }

    #[test]
    fn timer_upsert_overwrites() {
        let s = open();
        let n = s.create_note(0.0, 0.0, 240.0, 200.0).unwrap();
        let mut t = TimerRecord {
            note_id: n.id.clone(),
            mode: "countdown".into(),
            duration_seconds: Some(60),
            elapsed_seconds: 0,
            state: "running".into(),
            pomodoro_phase: None,
            started_at: Some(1),
            post_action_path: None,
            post_action_args: None,
            pomodoro_total_cycles: None,
            pomodoro_completed_cycles: None,
        };
        s.upsert_timer(&t).unwrap();
        t.elapsed_seconds = 30;
        t.state = "paused".into();
        s.upsert_timer(&t).unwrap();
        let got = s.get_timer(&n.id).unwrap().unwrap();
        assert_eq!(got.elapsed_seconds, 30);
        assert_eq!(got.state, "paused");
    }

    #[test]
    fn settings_set_get_overwrite() {
        let s = open();
        assert!(s.get_setting("missing").unwrap().is_none());

        s.set_setting("privacy_hide_from_capture", "true").unwrap();
        assert_eq!(
            s.get_setting("privacy_hide_from_capture").unwrap().unwrap(),
            "true"
        );

        s.set_setting("privacy_hide_from_capture", "false").unwrap();
        assert_eq!(
            s.get_setting("privacy_hide_from_capture").unwrap().unwrap(),
            "false"
        );
    }

    #[test]
    fn settings_list_returns_all_pairs() {
        let s = open();
        s.set_setting("a", "1").unwrap();
        s.set_setting("b", "two").unwrap();
        let mut all = s.list_settings().unwrap();
        all.sort();
        assert_eq!(
            all,
            vec![("a".into(), "1".into()), ("b".into(), "two".into())]
        );
    }
}
