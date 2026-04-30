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
            ",
        )?;
        // Add `text_color` column for older DBs that pre-date the feature.
        // SQLite has no `IF NOT EXISTS` for ADD COLUMN; ignoring the
        // duplicate-column error is the standard idiom.
        let _ = conn.execute("ALTER TABLE notes ADD COLUMN text_color TEXT", []);
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
        })
    }

    pub fn list_notes(&self) -> Result<Vec<NoteRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, content, color_id, opacity, always_on_top, x, y, width, height, created_at, updated_at, text_color
             FROM notes ORDER BY updated_at DESC",
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
            "SELECT id, content, color_id, opacity, always_on_top, x, y, width, height, created_at, updated_at, text_color
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

    pub fn upsert_timer(&self, t: &TimerRecord) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO timers (note_id, mode, duration_seconds, elapsed_seconds, state, pomodoro_phase, started_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(note_id) DO UPDATE SET
                mode = excluded.mode,
                duration_seconds = excluded.duration_seconds,
                elapsed_seconds = excluded.elapsed_seconds,
                state = excluded.state,
                pomodoro_phase = excluded.pomodoro_phase,
                started_at = excluded.started_at",
            params![
                t.note_id,
                t.mode,
                t.duration_seconds,
                t.elapsed_seconds,
                t.state,
                t.pomodoro_phase,
                t.started_at
            ],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_timer(&self, note_id: &str) -> Result<Option<TimerRecord>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT note_id, mode, duration_seconds, elapsed_seconds, state, pomodoro_phase, started_at
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
