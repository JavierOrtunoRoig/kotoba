use crate::models::{Item, Settings, Stats};
use anyhow::Result;
use chrono::{Local, NaiveDate, TimeZone};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Intervalos del SRS ligero. Se refinó en la Fase 2.
const INTERVALS_MINUTES: [i64; 8] = [10, 60, 360, 1440, 4320, 10080, 20160, 43200];

pub fn open(path: &Path) -> Result<Connection> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&mut conn)?;
    Ok(conn)
}

pub fn migrate(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            source_uid   TEXT NOT NULL UNIQUE,
            kanji        TEXT,
            reading      TEXT NOT NULL,
            meaning      TEXT NOT NULL,
            meaning_plain TEXT NOT NULL,
            kanji_gloss  TEXT,
            audio_file   TEXT,
            lesson       TEXT NOT NULL DEFAULT '',
            category     TEXT NOT NULL DEFAULT '',
            tags         TEXT NOT NULL DEFAULT '',
            is_pattern   INTEGER NOT NULL DEFAULT 0,
            irregular    INTEGER NOT NULL DEFAULT 0,
            dedup_key    TEXT NOT NULL UNIQUE,
            created_at   INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS item_sources (
            item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            source_uid  TEXT NOT NULL,
            lesson      TEXT NOT NULL DEFAULT '',
            category    TEXT NOT NULL DEFAULT '',
            PRIMARY KEY (item_id, source_uid)
        );

        CREATE TABLE IF NOT EXISTS review_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            shown_at    INTEGER NOT NULL,
            answered_at INTEGER,
            result      INTEGER NOT NULL,
            latency_ms  INTEGER NOT NULL DEFAULT 0,
            direction   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS srs_state (
            item_id          INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
            interval_minutes INTEGER NOT NULL DEFAULT 10,
            ease             REAL NOT NULL DEFAULT 2.5,
            due_at           INTEGER NOT NULL DEFAULT 0,
            reps             INTEGER NOT NULL DEFAULT 0,
            lapses           INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_srs_due ON srs_state(due_at);
        CREATE INDEX IF NOT EXISTS idx_review_answered ON review_log(answered_at);
        CREATE INDEX IF NOT EXISTS idx_items_pattern ON items(is_pattern);
        "#,
    )?;
    Ok(())
}

pub fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn today_start_ts() -> i64 {
    let now = Local::now();
    let start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| Local.from_local_datetime(&dt).single())
        .unwrap_or(now);
    start.timestamp()
}

pub fn count_items(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))?)
}

pub fn load_settings(conn: &Connection) -> Result<Settings> {
    let raw: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = 'settings'", [], |row| {
            row.get(0)
        })
        .optional()?;
    let settings = raw
        .and_then(|json| serde_json::from_str::<Settings>(&json).ok())
        .unwrap_or_default()
        .sanitize();
    Ok(settings)
}

pub fn save_settings(conn: &Connection, settings: &Settings) -> Result<Settings> {
    let settings = settings.clone().sanitize();
    let json = serde_json::to_string(&settings)?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('settings', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [json],
    )?;
    Ok(settings)
}

fn map_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    let audio_file: Option<String> = row.get(9)?;
    Ok(Item {
        id: row.get(0)?,
        kanji: row.get(1)?,
        reading: row.get(2)?,
        meaning: row.get(3)?,
        kanji_gloss: row.get(4)?,
        lesson: row.get(5)?,
        category: row.get(6)?,
        is_pattern: row.get::<_, i64>(7)? != 0,
        irregular: row.get::<_, i64>(8)? != 0,
        has_audio: audio_file.map(|f| !f.is_empty()).unwrap_or(false),
    })
}

const ITEM_COLUMNS: &str =
    "i.id, i.kanji, i.reading, i.meaning, i.kanji_gloss, i.lesson, i.category, i.is_pattern, i.irregular, i.audio_file";

pub fn item_by_id(conn: &Connection, id: i64) -> Result<Item> {
    let sql = format!("SELECT {ITEM_COLUMNS} FROM items i WHERE i.id = ?1");
    Ok(conn.query_row(&sql, [id], map_item)?)
}

/// Devuelve el siguiente ítem que toca repasar. Si no hay ninguno vencido,
/// cae a cualquier ítem para que «practicar ahora» siempre funcione.
pub fn next_item(
    conn: &Connection,
    exclude_id: i64,
    include_patterns: bool,
) -> Result<Option<Item>> {
    let due_sql = format!(
        r#"
        SELECT {ITEM_COLUMNS}
        FROM items i
        LEFT JOIN srs_state s ON s.item_id = i.id
        WHERE (?1 = 1 OR i.is_pattern = 0)
          AND i.id <> ?2
          AND (s.item_id IS NULL OR s.due_at <= ?3)
        ORDER BY COALESCE(s.due_at, 0) ASC, RANDOM()
        LIMIT 1
        "#
    );
    let found = conn
        .query_row(
            &due_sql,
            params![include_patterns as i32, exclude_id, now_ts()],
            map_item,
        )
        .optional()?;
    if found.is_some() {
        return Ok(found);
    }

    let any_sql = format!(
        r#"
        SELECT {ITEM_COLUMNS}
        FROM items i
        LEFT JOIN srs_state s ON s.item_id = i.id
        WHERE (?1 = 1 OR i.is_pattern = 0)
          AND i.id <> ?2
        ORDER BY COALESCE(s.due_at, 0) ASC, RANDOM()
        LIMIT 1
        "#
    );
    Ok(conn
        .query_row(&any_sql, params![include_patterns as i32, exclude_id], map_item)
        .optional()?)
}

pub fn due_count(conn: &Connection, include_patterns: bool) -> Result<i64> {
    Ok(conn.query_row(
        r#"
        SELECT COUNT(*)
        FROM items i
        LEFT JOIN srs_state s ON s.item_id = i.id
        WHERE (?1 = 1 OR i.is_pattern = 0)
          AND (s.item_id IS NULL OR s.due_at <= ?2)
        "#,
        params![include_patterns as i32, now_ts()],
        |row| row.get(0),
    )?)
}

pub fn reviews_today(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM review_log WHERE answered_at >= ?1",
        [today_start_ts()],
        |row| row.get(0),
    )?)
}

pub fn last_direction(conn: &Connection) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT direction FROM review_log ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?)
}

pub fn record_review(
    conn: &mut Connection,
    item_id: i64,
    correct: bool,
    direction: &str,
    latency_ms: i64,
) -> Result<i64> {
    let now = now_ts();
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO review_log (item_id, shown_at, answered_at, result, latency_ms, direction)
         VALUES (?1, ?2, ?2, ?3, ?4, ?5)",
        params![item_id, now, correct as i32, latency_ms.max(0), direction],
    )?;

    let current_reps: i64 = tx
        .query_row(
            "SELECT reps FROM srs_state WHERE item_id = ?1",
            [item_id],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0);

    let (reps, interval, lapses) = if correct {
        let reps = (current_reps + 1).min((INTERVALS_MINUTES.len() - 1) as i64);
        (reps, INTERVALS_MINUTES[reps as usize], 0)
    } else {
        (0, INTERVALS_MINUTES[0], 1)
    };

    let due_at = now + interval * 60;
    tx.execute(
        r#"
        INSERT INTO srs_state (item_id, interval_minutes, ease, due_at, reps, lapses)
        VALUES (?1, ?2, 2.5, ?3, ?4, ?5)
        ON CONFLICT(item_id) DO UPDATE SET
            interval_minutes = excluded.interval_minutes,
            due_at = excluded.due_at,
            reps = excluded.reps,
            lapses = srs_state.lapses + excluded.lapses
        "#,
        params![item_id, interval, due_at, reps, lapses],
    )?;

    tx.commit()?;
    Ok(interval)
}

pub fn stats(conn: &Connection) -> Result<Stats> {
    let total_items = count_items(conn)?;
    let total_reviews: i64 =
        conn.query_row("SELECT COUNT(*) FROM review_log", [], |row| row.get(0))?;
    let start = today_start_ts();
    let (reviewed_today, correct_today): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(result), 0) FROM review_log WHERE answered_at >= ?1",
        [start],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let accuracy_today = if reviewed_today > 0 {
        correct_today as f64 / reviewed_today as f64 * 100.0
    } else {
        0.0
    };
    let mastered: i64 = conn.query_row(
        "SELECT COUNT(*) FROM srs_state WHERE interval_minutes >= 10080",
        [],
        |row| row.get(0),
    )?;
    let due_now: i64 = conn.query_row(
        r#"
        SELECT COUNT(*) FROM items i
        LEFT JOIN srs_state s ON s.item_id = i.id
        WHERE s.item_id IS NULL OR s.due_at <= ?1
        "#,
        [now_ts()],
        |row| row.get(0),
    )?;

    Ok(Stats {
        total_items,
        total_reviews,
        reviewed_today,
        correct_today,
        accuracy_today,
        streak_days: streak_days(conn)?,
        mastered,
        due_now,
        activity: activity(conn)?,
    })
}

fn activity(conn: &Connection) -> Result<Vec<crate::models::DayCount>> {
    use chrono::Duration;

    let today = Local::now().date_naive();
    let start = today - Duration::days(13);
    let since = start
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| Local.from_local_datetime(&dt).single())
        .map(|dt| dt.timestamp())
        .unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT date(answered_at, 'unixepoch', 'localtime') AS day,
                COUNT(*), COALESCE(SUM(result), 0)
         FROM review_log
         WHERE answered_at >= ?1
         GROUP BY day",
    )?;
    let rows = stmt.query_map([since], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut counts: HashMap<String, (i64, i64)> = HashMap::new();
    for row in rows {
        let (day, total, correct) = row?;
        counts.insert(day, (total, correct));
    }

    let mut result = Vec::with_capacity(14);
    for offset in 0..14 {
        let day = start + Duration::days(offset);
        let key = day.format("%Y-%m-%d").to_string();
        let (total, correct) = counts.get(&key).copied().unwrap_or((0, 0));
        result.push(crate::models::DayCount {
            date: key,
            total,
            correct,
        });
    }
    Ok(result)
}

fn streak_days(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT date(answered_at, 'unixepoch', 'localtime')
         FROM review_log ORDER BY 1 DESC LIMIT 400",
    )?;
    let days: HashSet<NaiveDate> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|row| row.ok())
        .filter_map(|value| NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok())
        .collect();

    if days.is_empty() {
        return Ok(0);
    }

    let today = Local::now().date_naive();
    let mut cursor = if days.contains(&today) {
        today
    } else {
        today.pred_opt().unwrap_or(today)
    };
    let mut streak = 0;
    while days.contains(&cursor) {
        streak += 1;
        match cursor.pred_opt() {
            Some(previous) => cursor = previous,
            None => break,
        }
    }
    Ok(streak)
}

pub fn reset_progress(conn: &Connection) -> Result<()> {
    conn.execute_batch("DELETE FROM review_log; DELETE FROM srs_state;")?;
    Ok(())
}

pub fn clear_library(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM review_log;
         DELETE FROM srs_state;
         DELETE FROM item_sources;
         DELETE FROM items;",
    )?;
    Ok(())
}

/// Datos para insertar un ítem desde el importador.
pub struct NewItem<'a> {
    pub source_uid: &'a str,
    pub kanji: Option<&'a str>,
    pub reading: &'a str,
    pub meaning: &'a str,
    pub meaning_plain: &'a str,
    pub kanji_gloss: Option<&'a str>,
    pub audio_file: Option<&'a str>,
    pub lesson: &'a str,
    pub category: &'a str,
    pub tags: &'a str,
    pub is_pattern: bool,
    pub irregular: bool,
    pub dedup_key: &'a str,
}

pub enum InsertOutcome {
    Inserted,
    Merged,
    Skipped,
}

pub fn insert_item(tx: &rusqlite::Transaction<'_>, item: &NewItem<'_>) -> Result<InsertOutcome> {
    let existing: Option<i64> = tx
        .query_row(
            "SELECT id FROM items WHERE source_uid = ?1",
            [item.source_uid],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        tx.execute(
            "INSERT OR IGNORE INTO item_sources (item_id, source_uid, lesson, category)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, item.source_uid, item.lesson, item.category],
        )?;
        return Ok(InsertOutcome::Skipped);
    }

    tx.execute(
        "INSERT INTO items
         (source_uid, kanji, reading, meaning, meaning_plain, kanji_gloss, audio_file,
          lesson, category, tags, is_pattern, irregular, dedup_key, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(dedup_key) DO NOTHING",
        params![
            item.source_uid,
            item.kanji,
            item.reading,
            item.meaning,
            item.meaning_plain,
            item.kanji_gloss,
            item.audio_file,
            item.lesson,
            item.category,
            item.tags,
            item.is_pattern as i32,
            item.irregular as i32,
            item.dedup_key,
            now_ts(),
        ],
    )?;

    let inserted = tx.changes() > 0;
    let id: i64 = if inserted {
        tx.last_insert_rowid()
    } else {
        tx.query_row(
            "SELECT id FROM items WHERE dedup_key = ?1",
            [item.dedup_key],
            |row| row.get(0),
        )?
    };

    tx.execute(
        "INSERT OR IGNORE INTO item_sources (item_id, source_uid, lesson, category)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, item.source_uid, item.lesson, item.category],
    )?;

    Ok(if inserted {
        InsertOutcome::Inserted
    } else {
        InsertOutcome::Merged
    })
}
