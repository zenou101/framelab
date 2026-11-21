use crate::types::{GameEntry, SessionSummary};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

const SCHEMA_VERSION: i64 = 1;

pub struct NewSession {
    pub game_id: Option<i64>,
    pub game_name: String,
    pub started_at_ms: i64,
    pub primary_source: String,
    pub hardware_json: String,
    pub meta_json: String,
}

pub struct FinishedSession {
    pub ended_at_ms: i64,
    pub duration_ms: i64,
    pub status: String,
    pub recovered: bool,
    pub frames_displayed: Option<i64>,
    pub frames_presented: Option<i64>,
    pub avg_fps_displayed: Option<f64>,
    pub avg_fps_presented: Option<f64>,
    pub median_fps_displayed: Option<f64>,
    pub p1_low_displayed: Option<f64>,
    pub p01_low_displayed: Option<f64>,
    pub p1_low_presented: Option<f64>,
    pub stutter_count: Option<i64>,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let path = dir.join("framelab.db");
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.pragma_update(None, "journal_mode", "WAL").map_err(|e| e.to_string())?;
        conn.pragma_update(None, "synchronous", "NORMAL").map_err(|e| e.to_string())?;
        conn.pragma_update(None, "foreign_keys", "ON").map_err(|e| e.to_string())?;
        let db = Self { conn };
        db.migrate()?;
        db.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").ok();
        Ok(db)
    }

    fn migrate(&self) -> Result<(), String> {
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        if version >= SCHEMA_VERSION {
            return Ok(());
        }
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS games (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    exe_path TEXT,
                    source TEXT NOT NULL DEFAULT 'manual',
                    launcher_id TEXT,
                    icon_path TEXT,
                    created_at_ms INTEGER NOT NULL
                );
                CREATE TABLE IF NOT EXISTS sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    game_id INTEGER REFERENCES games(id) ON DELETE SET NULL,
                    game_name TEXT NOT NULL,
                    started_at_ms INTEGER NOT NULL,
                    ended_at_ms INTEGER,
                    duration_ms INTEGER NOT NULL DEFAULT 0,
                    status TEXT NOT NULL DEFAULT 'recording',
                    recovered INTEGER NOT NULL DEFAULT 0,
                    primary_source TEXT NOT NULL DEFAULT 'displayed',
                    frames_displayed INTEGER,
                    frames_presented INTEGER,
                    avg_fps_displayed REAL,
                    avg_fps_presented REAL,
                    median_fps_displayed REAL,
                    median_fps_presented REAL,
                    p1_low_displayed REAL,
                    p01_low_displayed REAL,
                    p1_low_presented REAL,
                    stutter_count INTEGER,
                    hardware_json TEXT,
                    meta_json TEXT,
                    notes TEXT
                );
                CREATE TABLE IF NOT EXISTS session_frames (
                    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                    kind TEXT NOT NULL,
                    chunk INTEGER NOT NULL,
                    t0_ms INTEGER NOT NULL,
                    count INTEGER NOT NULL,
                    data BLOB NOT NULL,
                    PRIMARY KEY (session_id, kind, chunk)
                );
                CREATE TABLE IF NOT EXISTS session_samples (
                    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                    chunk INTEGER NOT NULL,
                    t0_ms INTEGER NOT NULL,
                    data BLOB NOT NULL,
                    PRIMARY KEY (session_id, chunk)
                );
                CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at_ms DESC);
                CREATE INDEX IF NOT EXISTS idx_sessions_game ON sessions(game_id);
                PRAGMA user_version = 1;
                "#,
            )
            .map_err(|e| e.to_string())
    }

    /// Any session still marked `recording` belongs to a process that died.
    /// Mark it interrupted and let the caller rebuild its statistics from the
    /// chunks that were already flushed.
    pub fn recover_interrupted(&self) -> Vec<i64> {
        let ids: Vec<i64> = self
            .conn
            .prepare("SELECT id FROM sessions WHERE status = 'recording'")
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
                rows.collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default();
        for id in &ids {
            let _ = self.conn.execute(
                "UPDATE sessions SET status = 'interrupted', recovered = 1 WHERE id = ?1",
                params![id],
            );
        }
        ids
    }

    pub fn create_session(&self, session: &NewSession) -> Result<i64, String> {
        self.conn
            .execute(
                "INSERT INTO sessions (game_id, game_name, started_at_ms, status, primary_source, hardware_json, meta_json)
                 VALUES (?1, ?2, ?3, 'recording', ?4, ?5, ?6)",
                params![
                    session.game_id,
                    session.game_name,
                    session.started_at_ms,
                    session.primary_source,
                    session.hardware_json,
                    session.meta_json
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn append_frames(&self, session_id: i64, kind: &str, chunk: i64, t0_ms: i64, frames: &[f64]) -> Result<(), String> {
        if frames.is_empty() {
            return Ok(());
        }
        let mut bytes = Vec::with_capacity(frames.len() * 4);
        for value in frames {
            bytes.extend_from_slice(&(*value as f32).to_le_bytes());
        }
        self.conn
            .execute(
                "INSERT OR REPLACE INTO session_frames (session_id, kind, chunk, t0_ms, count, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![session_id, kind, chunk, t0_ms, frames.len() as i64, bytes],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_frames(&self, session_id: i64, kind: &str) -> Result<Vec<f64>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM session_frames WHERE session_id = ?1 AND kind = ?2 ORDER BY chunk")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![session_id, kind], |row| row.get::<_, Vec<u8>>(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for blob in rows.flatten() {
            for chunk in blob.chunks_exact(4) {
                let bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
                out.push(f32::from_le_bytes(bytes) as f64);
            }
        }
        Ok(out)
    }

    pub fn finish_session(&self, id: i64, data: &FinishedSession) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE sessions SET ended_at_ms = ?2, duration_ms = ?3, status = ?4, recovered = ?5,
                 frames_displayed = ?6, frames_presented = ?7, avg_fps_displayed = ?8, avg_fps_presented = ?9,
                 median_fps_displayed = ?10, p1_low_displayed = ?11, p01_low_displayed = ?12,
                 p1_low_presented = ?13, stutter_count = ?14 WHERE id = ?1",
                params![
                    id,
                    data.ended_at_ms,
                    data.duration_ms,
                    data.status,
                    data.recovered as i64,
                    data.frames_displayed,
                    data.frames_presented,
                    data.avg_fps_displayed,
                    data.avg_fps_presented,
                    data.median_fps_displayed,
                    data.p1_low_displayed,
                    data.p01_low_displayed,
                    data.p1_low_presented,
                    data.stutter_count
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn session(&self, id: i64) -> Option<SessionSummary> {
        self.conn
            .query_row(
                &format!("SELECT {SESSION_COLUMNS} FROM sessions WHERE id = ?1"),
                params![id],
                map_session,
            )
            .optional()
            .ok()
            .flatten()
    }

    pub fn list_sessions(&self, limit: i64, offset: i64, game_id: Option<i64>, search: Option<&str>) -> Vec<SessionSummary> {
        let pattern = search.map(|value| format!("%{}%", value.replace('%', "")));
        let sql = format!(
            "SELECT {SESSION_COLUMNS} FROM sessions
             WHERE (?1 IS NULL OR game_id = ?1) AND (?2 IS NULL OR game_name LIKE ?2)
             ORDER BY started_at_ms DESC LIMIT ?3 OFFSET ?4"
        );
        let Ok(mut stmt) = self.conn.prepare(&sql) else { return Vec::new() };
        stmt.query_map(params![game_id, pattern, limit.clamp(1, 500), offset.max(0)], map_session)
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    pub fn delete_session(&self, id: i64) -> Result<(), String> {
        self.conn.execute("DELETE FROM sessions WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn purge_older_than(&self, cutoff_ms: i64) -> Result<usize, String> {
        self.conn
            .execute("DELETE FROM sessions WHERE started_at_ms < ?1", params![cutoff_ms])
            .map_err(|e| e.to_string())
    }

    pub fn clear_sessions(&self) -> Result<(), String> {
        self.conn.execute("DELETE FROM sessions", []).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn counts(&self) -> (i64, i64) {
        let sessions = self.conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get::<_, i64>(0)).unwrap_or(0);
        let games = self.conn.query_row("SELECT COUNT(*) FROM games", [], |row| row.get::<_, i64>(0)).unwrap_or(0);
        (sessions, games)
    }

    pub fn games(&self) -> Vec<GameEntry> {
        let sql = "SELECT g.id, g.name, g.exe_path, g.source, g.launcher_id, g.icon_path, g.created_at_ms,
                   (SELECT COUNT(*) FROM sessions s WHERE s.game_id = g.id),
                   (SELECT MAX(s.avg_fps_displayed) FROM sessions s WHERE s.game_id = g.id)
                   FROM games g ORDER BY g.name COLLATE NOCASE";
        let Ok(mut stmt) = self.conn.prepare(sql) else { return Vec::new() };
        stmt.query_map([], |row| {
            Ok(GameEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                exe_path: row.get(2)?,
                source: row.get(3)?,
                launcher_id: row.get(4)?,
                icon_path: row.get(5)?,
                created_at_ms: row.get(6)?,
                session_count: row.get(7)?,
                best_avg_fps: row.get(8)?,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    pub fn insert_game(&self, name: &str, exe_path: Option<&str>, source: &str, launcher_id: Option<&str>, icon_path: Option<&str>) -> Result<i64, String> {
        if let (Some(exe), Some(launcher)) = (exe_path, launcher_id) {
            let existing: Option<i64> = self
                .conn
                .query_row("SELECT id FROM games WHERE exe_path = ?1 OR launcher_id = ?2", params![exe, launcher], |row| row.get(0))
                .optional()
                .map_err(|e| e.to_string())?;
            if let Some(id) = existing {
                return Ok(id);
            }
        }
        self.conn
            .execute(
                "INSERT INTO games (name, exe_path, source, launcher_id, icon_path, created_at_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![name, exe_path, source, launcher_id, icon_path, crate::sensors::now_ms()],
            )
            .map_err(|e| e.to_string())?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_game(&self, id: i64) -> Result<(), String> {
        self.conn.execute("DELETE FROM games WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }
}

const SESSION_COLUMNS: &str = "id, game_id, game_name, started_at_ms, ended_at_ms, duration_ms, status, primary_source,
    frames_displayed, frames_presented, avg_fps_displayed, avg_fps_presented, median_fps_displayed,
    p1_low_displayed, p01_low_displayed, p1_low_presented, stutter_count, recovered, hardware_json, meta_json, notes";

fn map_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionSummary> {
    Ok(SessionSummary {
        id: row.get(0)?,
        game_id: row.get(1)?,
        game_name: row.get(2)?,
        started_at_ms: row.get(3)?,
        ended_at_ms: row.get(4)?,
        duration_ms: row.get(5)?,
        status: row.get(6)?,
        primary_source: row.get(7)?,
        frames_displayed: row.get(8)?,
        frames_presented: row.get(9)?,
        avg_fps_displayed: row.get(10)?,
        avg_fps_presented: row.get(11)?,
        median_fps_displayed: row.get(12)?,
        p1_low_displayed: row.get(13)?,
        p01_low_displayed: row.get(14)?,
        p1_low_presented: row.get(15)?,
        stutter_count: row.get(16)?,
        recovered: row.get::<_, i64>(17)? != 0,
        hardware_json: row.get(18)?,
        meta_json: row.get(19)?,
        notes: row.get(20)?,
    })
}

impl Db {
    pub fn append_samples(&self, session_id: i64, chunk: i64, t0_ms: i64, bytes: &[u8]) -> Result<(), String> {
        if bytes.is_empty() {
            return Ok(());
        }
        self.conn
            .execute(
                "INSERT OR REPLACE INTO session_samples (session_id, chunk, t0_ms, data) VALUES (?1, ?2, ?3, ?4)",
                params![session_id, chunk, t0_ms, bytes],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_samples(&self, session_id: i64) -> Vec<u8> {
        let Ok(mut stmt) = self
            .conn
            .prepare("SELECT data FROM session_samples WHERE session_id = ?1 ORDER BY chunk")
        else {
            return Vec::new();
        };
        let rows = stmt.query_map(params![session_id], |row| row.get::<_, Vec<u8>>(0));
        let mut out = Vec::new();
        if let Ok(rows) = rows {
            for blob in rows.flatten() {
                out.extend_from_slice(&blob);
            }
        }
        out
    }
}
