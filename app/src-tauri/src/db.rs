//! The student's progress in a local SQLite file: profile, attempts, steps, journal,
//! unlocked achievements and where each lesson was left off.

use std::collections::HashMap;
use std::path::Path;

use chrono::NaiveDateTime;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::error::AppResult;

const TIME: &str = "%Y-%m-%dT%H:%M:%S";

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    name TEXT NOT NULL DEFAULT '',
    daily_goal INTEGER NOT NULL DEFAULT 2,
    created TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS attempt (
    id INTEGER PRIMARY KEY,
    lesson TEXT NOT NULL,
    started TEXT NOT NULL,
    finished TEXT,
    passed INTEGER NOT NULL DEFAULT 0,
    hints INTEGER NOT NULL DEFAULT 0,
    seconds INTEGER NOT NULL DEFAULT 0,
    xp INTEGER NOT NULL DEFAULT 0,
    similarity REAL
);
CREATE INDEX IF NOT EXISTS attempt_lesson ON attempt (lesson);
CREATE TABLE IF NOT EXISTS step (
    attempt INTEGER NOT NULL REFERENCES attempt (id),
    step TEXT NOT NULL,
    done TEXT NOT NULL,
    hinted INTEGER NOT NULL,
    PRIMARY KEY (attempt, step)
);
CREATE TABLE IF NOT EXISTS journal (
    id INTEGER PRIMARY KEY,
    at TEXT NOT NULL,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS achievement (
    id TEXT PRIMARY KEY,
    at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS stage (
    lesson TEXT PRIMARY KEY,
    stage TEXT NOT NULL,
    at TEXT NOT NULL
);
";

pub fn fmt(t: NaiveDateTime) -> String {
    t.format(TIME).to_string()
}

pub fn parse(text: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(text, TIME).ok()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub daily_goal: u32,
    pub created: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub id: i64,
    pub lesson: String,
    pub started: NaiveDateTime,
    pub finished: Option<NaiveDateTime>,
    pub passed: bool,
    pub hints: u32,
    pub seconds: u32,
    pub xp: u32,
    pub similarity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub at: String,
    pub kind: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub lesson: String,
    pub stage: String,
    pub at: String,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &Path, now: NaiveDateTime) -> AppResult<Db> {
        Db::init(Connection::open(path)?, now)
    }

    pub fn in_memory(now: NaiveDateTime) -> AppResult<Db> {
        Db::init(Connection::open_in_memory()?, now)
    }

    fn init(conn: Connection, now: NaiveDateTime) -> AppResult<Db> {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        conn.execute_batch(SCHEMA)?;
        conn.execute("INSERT OR IGNORE INTO profile (id, created) VALUES (1, ?1)", params![fmt(now)])?;
        Ok(Db { conn })
    }

    pub fn profile(&self) -> AppResult<Profile> {
        Ok(self.conn.query_row("SELECT name, daily_goal, created FROM profile WHERE id = 1", [], |r| {
            Ok(Profile { name: r.get(0)?, daily_goal: r.get(1)?, created: r.get(2)? })
        })?)
    }

    pub fn set_profile(&self, name: &str, daily_goal: u32) -> AppResult<()> {
        self.conn.execute("UPDATE profile SET name = ?1, daily_goal = ?2 WHERE id = 1", params![name.trim(), daily_goal.clamp(1, 10)])?;
        Ok(())
    }

    pub fn start_attempt(&self, lesson: &str, now: NaiveDateTime) -> AppResult<i64> {
        self.conn.execute("INSERT INTO attempt (lesson, started) VALUES (?1, ?2)", params![lesson, fmt(now)])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn add_hint(&self, attempt: i64) -> AppResult<()> {
        self.conn.execute("UPDATE attempt SET hints = hints + 1 WHERE id = ?1", params![attempt])?;
        Ok(())
    }

    pub fn record_step(&self, attempt: i64, step: &str, hinted: bool, now: NaiveDateTime) -> AppResult<()> {
        self.conn.execute("INSERT OR IGNORE INTO step (attempt, step, done, hinted) VALUES (?1, ?2, ?3, ?4)", params![attempt, step, fmt(now), hinted])?;
        Ok(())
    }

    pub fn finish_attempt(&self, attempt: i64, passed: bool, seconds: u32, xp: u32, similarity: Option<f64>, now: NaiveDateTime) -> AppResult<()> {
        self.conn.execute(
            "UPDATE attempt SET finished = ?2, passed = ?3, seconds = ?4, xp = ?5, similarity = ?6 WHERE id = ?1",
            params![attempt, fmt(now), passed, seconds, xp, similarity],
        )?;
        Ok(())
    }

    pub fn attempts(&self) -> AppResult<Vec<Attempt>> {
        let mut stmt = self.conn.prepare("SELECT id, lesson, started, finished, passed, hints, seconds, xp, similarity FROM attempt ORDER BY started, id")?;
        let rows = stmt.query_map([], |r| {
            let started: String = r.get(2)?;
            let finished: Option<String> = r.get(3)?;
            Ok((r.get(0)?, r.get(1)?, started, finished, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, lesson, started, finished, passed, hints, seconds, xp, similarity) = row?;
            let Some(started) = parse(&started) else { continue };
            out.push(Attempt { id, lesson, started, finished: finished.as_deref().and_then(parse), passed, hints, seconds, xp, similarity });
        }
        Ok(out)
    }

    pub fn attempt(&self, id: i64) -> AppResult<Option<Attempt>> {
        Ok(self.attempts()?.into_iter().find(|a| a.id == id))
    }

    /// Steps of an attempt with whether a hint was used on them.
    pub fn steps(&self, attempt: i64) -> AppResult<Vec<(String, bool)>> {
        let mut stmt = self.conn.prepare("SELECT step, hinted FROM step WHERE attempt = ?1 ORDER BY rowid")?;
        let rows = stmt.query_map(params![attempt], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn log(&self, now: NaiveDateTime, kind: &str, title: &str, detail: &str) -> AppResult<()> {
        self.conn.execute("INSERT INTO journal (at, kind, title, detail) VALUES (?1, ?2, ?3, ?4)", params![fmt(now), kind, title, detail])?;
        Ok(())
    }

    pub fn journal(&self, limit: u32) -> AppResult<Vec<JournalEntry>> {
        let mut stmt = self.conn.prepare("SELECT at, kind, title, detail FROM journal ORDER BY at DESC, id DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit], |r| Ok(JournalEntry { at: r.get(0)?, kind: r.get(1)?, title: r.get(2)?, detail: r.get(3)? }))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn unlocked(&self) -> AppResult<HashMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT id, at FROM achievement")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Stores an achievement; returns whether it was new.
    pub fn unlock(&self, id: &str, now: NaiveDateTime) -> AppResult<bool> {
        Ok(self.conn.execute("INSERT OR IGNORE INTO achievement (id, at) VALUES (?1, ?2)", params![id, fmt(now)])? > 0)
    }

    pub fn set_stage(&self, lesson: &str, stage: &str, now: NaiveDateTime) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO stage (lesson, stage, at) VALUES (?1, ?2, ?3) ON CONFLICT (lesson) DO UPDATE SET stage = ?2, at = ?3",
            params![lesson, stage, fmt(now)],
        )?;
        Ok(())
    }

    pub fn clear_stage(&self, lesson: &str) -> AppResult<()> {
        self.conn.execute("DELETE FROM stage WHERE lesson = ?1", params![lesson])?;
        Ok(())
    }

    /// The most recently touched unfinished lesson.
    pub fn latest_stage(&self) -> AppResult<Option<Stage>> {
        Ok(self
            .conn
            .query_row("SELECT lesson, stage, at FROM stage ORDER BY at DESC LIMIT 1", [], |r| Ok(Stage { lesson: r.get(0)?, stage: r.get(1)?, at: r.get(2)? }))
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn at(day: u32, hour: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, day).and_then(|d| d.and_hms_opt(hour, 0, 0)).unwrap()
    }

    #[test]
    fn a_fresh_database_has_an_empty_profile() {
        let db = Db::in_memory(at(1, 9)).unwrap();
        let p = db.profile().unwrap();
        assert_eq!(p.name, "");
        assert_eq!(p.daily_goal, 2);
        assert_eq!(p.created, "2026-10-01T09:00:00");
        db.set_profile("  Алекс ", 50).unwrap();
        let p = db.profile().unwrap();
        assert_eq!(p.name, "Алекс");
        assert_eq!(p.daily_goal, 10, "the goal is clamped");
        assert!(db.attempts().unwrap().is_empty());
        assert!(db.latest_stage().unwrap().is_none());
    }

    #[test]
    fn attempts_record_steps_hints_and_results() {
        let db = Db::in_memory(at(1, 9)).unwrap();
        let id = db.start_attempt("light.cast-shadow", at(2, 10)).unwrap();
        db.add_hint(id).unwrap();
        db.add_hint(id).unwrap();
        db.record_step(id, "layer", false, at(2, 10)).unwrap();
        db.record_step(id, "cast", true, at(2, 11)).unwrap();
        db.record_step(id, "cast", false, at(2, 12)).unwrap();
        db.finish_attempt(id, true, 420, 150, Some(0.91), at(2, 12)).unwrap();
        let a = db.attempt(id).unwrap().unwrap();
        assert!(a.passed);
        assert_eq!((a.hints, a.seconds, a.xp), (2, 420, 150));
        assert_eq!(a.finished, Some(at(2, 12)));
        assert_eq!(a.similarity, Some(0.91));
        assert_eq!(db.steps(id).unwrap(), vec![("layer".to_string(), false), ("cast".to_string(), true)], "a step is recorded once");
    }

    #[test]
    fn journal_achievements_and_stages() {
        let db = Db::in_memory(at(1, 9)).unwrap();
        db.log(at(3, 8), "lesson", "Начат урок", "").unwrap();
        db.log(at(3, 9), "pass", "Пройден урок", "+60 XP").unwrap();
        let j = db.journal(10).unwrap();
        assert_eq!(j[0].title, "Пройден урок");
        assert_eq!(j.len(), 2);
        assert!(db.unlock("first", at(3, 9)).unwrap());
        assert!(!db.unlock("first", at(4, 9)).unwrap());
        assert_eq!(db.unlocked().unwrap().get("first").map(String::as_str), Some("2026-10-03T09:00:00"));
        db.set_stage("a.one", "theory", at(3, 9)).unwrap();
        db.set_stage("a.two", "guide", at(3, 10)).unwrap();
        db.set_stage("a.one", "practice", at(3, 11)).unwrap();
        assert_eq!(db.latest_stage().unwrap().unwrap().lesson, "a.one");
        db.clear_stage("a.one").unwrap();
        assert_eq!(db.latest_stage().unwrap().unwrap().stage, "guide");
    }

    #[test]
    fn the_file_survives_reopening() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fude.db");
        {
            let db = Db::open(&path, at(1, 9)).unwrap();
            db.set_profile("Аня", 3).unwrap();
            db.start_attempt("x.y", at(1, 10)).unwrap();
        }
        let db = Db::open(&path, at(5, 9)).unwrap();
        assert_eq!(db.profile().unwrap().name, "Аня");
        assert_eq!(db.profile().unwrap().created, "2026-10-01T09:00:00", "created is kept");
        assert_eq!(db.attempts().unwrap().len(), 1);
    }
}
