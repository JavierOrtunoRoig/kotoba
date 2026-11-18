use crate::models::{Item, Settings, Stats};
use anyhow::Result;
use chrono::{Local, NaiveDate, TimeZone};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Intervalos de aprendizaje antes de entrar en el ciclo SM-2.
const LEARNING_INTERVAL_MINUTES: i64 = 10;
const FIRST_REVIEW_INTERVAL_MINUTES: i64 = 1440;
pub const MAX_QUALITY: u8 = 5;

/// Calcula el siguiente estado SM-2.
///
/// `quality`: 0-5 (5 = acierto rápido, 4 = acierto, 2 = fallo, 1 = no lo sé).
/// Devuelve `(reps, interval_minutes, ease)`.
pub fn sm2_next(
    quality: u8,
    reps: i64,
    interval_minutes: i64,
    ease: f64,
) -> (i64, i64, f64) {
    let q = quality.min(MAX_QUALITY) as f64;
    let ease = (ease + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02))).max(1.3);

    if quality < 3 {
        return (0, LEARNING_INTERVAL_MINUTES, ease);
    }

    let (reps, interval) = match reps {
        0 => (1, LEARNING_INTERVAL_MINUTES),
        1 => (2, FIRST_REVIEW_INTERVAL_MINUTES),
        _ => {
            let days = (interval_minutes as f64 / 1440.0) * ease;
            let interval = (days * 1440.0).round().max(1440.0) as i64;
            (reps + 1, interval)
        }
    };

    (reps, interval, ease)
}

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
            quality     INTEGER NOT NULL DEFAULT 0,
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

    // Migraciones incrementales. `user_version` empieza en 0 en bases nuevas y antiguas.
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version < 1 {
        ensure_column(conn, "review_log", "quality", "INTEGER NOT NULL DEFAULT 0")?;
        conn.pragma_update(None, "user_version", 1)?;
    }

    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
        params![table, column],
        |row| row.get(0),
    )?;
    if exists == 0 {
        conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"))?;
    }
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

#[cfg(test)]
pub fn all_items(conn: &Connection) -> Result<Vec<Item>> {
    let sql = format!("SELECT {ITEM_COLUMNS} FROM items i ORDER BY i.id");
    let mut stmt = conn.prepare(&sql)?;
    let items = stmt
        .query_map([], map_item)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(items)
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
    quality: u8,
    direction: &str,
    latency_ms: i64,
) -> Result<i64> {
    let now = now_ts();
    let correct = quality >= 3;
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO review_log
         (item_id, shown_at, answered_at, result, quality, latency_ms, direction)
         VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6)",
        params![
            item_id,
            now,
            correct as i32,
            quality as i32,
            latency_ms.max(0),
            direction
        ],
    )?;

    let (current_reps, current_interval, current_ease): (i64, i64, f64) = tx
        .query_row(
            "SELECT reps, interval_minutes, ease FROM srs_state WHERE item_id = ?1",
            [item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .unwrap_or((0, 0, 2.5));

    let (reps, interval, ease) = sm2_next(quality, current_reps, current_interval, current_ease);
    let lapses = if quality < 3 { 1 } else { 0 };
    let due_at = now + interval * 60;

    tx.execute(
        r#"
        INSERT INTO srs_state (item_id, interval_minutes, ease, due_at, reps, lapses)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(item_id) DO UPDATE SET
            interval_minutes = excluded.interval_minutes,
            ease = excluded.ease,
            due_at = excluded.due_at,
            reps = excluded.reps,
            lapses = srs_state.lapses + excluded.lapses
        "#,
        params![item_id, interval, ease, due_at, reps, lapses],
    )?;

    tx.commit()?;
    Ok(interval)
}

pub fn stats(conn: &Connection, include_patterns: bool) -> Result<Stats> {
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
        WHERE (?1 = 1 OR i.is_pattern = 0)
          AND (s.item_id IS NULL OR s.due_at <= ?2)
        "#,
        params![include_patterns as i32, now_ts()],
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
        lessons: lessons(conn)?,
        difficult: difficult(conn)?,
    })
}

fn lessons(conn: &Connection) -> Result<Vec<crate::models::LessonStat>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT i.lesson,
               COUNT(DISTINCT i.id) AS total,
               COUNT(DISTINCT CASE WHEN s.item_id IS NOT NULL THEN i.id END) AS reviewed,
               COUNT(DISTINCT CASE WHEN s.interval_minutes >= 10080 THEN i.id END) AS mastered,
               COUNT(r.id) AS reviews,
               COALESCE(SUM(r.result), 0) AS correct
        FROM items i
        LEFT JOIN srs_state s ON s.item_id = i.id
        LEFT JOIN review_log r ON r.item_id = i.id
        GROUP BY i.lesson
        ORDER BY i.lesson
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (lesson, total, reviewed, mastered, reviews, correct) = row?;
        let accuracy = if reviews > 0 {
            correct as f64 / reviews as f64 * 100.0
        } else {
            0.0
        };
        out.push(crate::models::LessonStat {
            lesson: if lesson.is_empty() {
                "Sin lección".to_string()
            } else {
                lesson
            },
            total,
            reviewed,
            mastered,
            reviews,
            correct,
            accuracy,
        });
    }
    Ok(out)
}

fn difficult(conn: &Connection) -> Result<Vec<crate::models::DifficultItem>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT i.id, i.kanji, i.reading, i.meaning,
               COUNT(r.id) AS attempts,
               SUM(CASE WHEN r.result = 0 THEN 1 ELSE 0 END) AS failures,
               COALESCE(s.lapses, 0) AS lapses
        FROM items i
        JOIN review_log r ON r.item_id = i.id
        LEFT JOIN srs_state s ON s.item_id = i.id
        GROUP BY i.id
        HAVING failures > 0
        ORDER BY failures DESC, attempts DESC
        LIMIT 5
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (id, kanji, reading, meaning, attempts, failures, lapses) = row?;
        let japanese = kanji
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(reading);
        out.push(crate::models::DifficultItem {
            id,
            japanese,
            meaning,
            attempts,
            failures,
            lapses,
        });
    }
    Ok(out)
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

/// Borra los archivos de `media/` que ya no referencia ningún ítem.
pub fn cleanup_media(conn: &Connection, media_dir: &Path) -> Result<usize> {
    let mut stmt =
        conn.prepare("SELECT audio_file FROM items WHERE audio_file IS NOT NULL AND audio_file <> ''")?;
    let referenced: HashSet<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|row| row.ok())
        .collect();

    if !media_dir.exists() {
        return Ok(0);
    }

    let mut removed = 0usize;
    for entry in std::fs::read_dir(media_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !referenced.contains(&name) {
            std::fs::remove_file(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sm2_learning_step() {
        let (reps, interval, ease) = sm2_next(5, 0, 0, 2.5);
        assert_eq!(reps, 1);
        assert_eq!(interval, 10);
        assert!((ease - 2.6).abs() < 1e-9);
    }

    #[test]
    fn sm2_first_day_and_growth() {
        let (reps, interval, ease) = sm2_next(4, 1, 10, 2.5);
        assert_eq!((reps, interval), (2, 1440));
        assert!((ease - 2.5).abs() < 1e-9);

        let (reps, interval, ease) = sm2_next(4, 2, 1440, 2.5);
        assert_eq!(reps, 3);
        assert_eq!(interval, 3600);
        assert!((ease - 2.5).abs() < 1e-9);
    }

    #[test]
    fn sm2_lapse_resets() {
        let (reps, interval, ease) = sm2_next(2, 4, 10080, 2.5);
        assert_eq!((reps, interval), (0, 10));
        assert!((ease - 2.18).abs() < 1e-9);
    }

    #[test]
    fn sm2_ease_floor() {
        let (_, _, ease) = sm2_next(0, 3, 1440, 1.35);
        assert!((ease - 1.3).abs() < 1e-9);
    }

    #[test]
    fn cleans_unreferenced_media() {
        let mut conn = Connection::open_in_memory().expect("db");
        migrate(&mut conn).expect("migrate");
        let dir = tempfile::tempdir().expect("tempdir");

        std::fs::write(dir.path().join("keep.mp3"), b"keep").expect("keep");
        std::fs::write(dir.path().join("orphan.mp3"), b"orphan").expect("orphan");

        conn.execute(
            "INSERT INTO items
             (source_uid, reading, meaning, meaning_plain, audio_file, dedup_key, created_at)
             VALUES ('u1', 'a', 'b', 'b', 'keep.mp3', 'a|b', 0)",
            [],
        )
        .expect("item");

        let removed = cleanup_media(&conn, dir.path()).expect("cleanup");
        assert_eq!(removed, 1);
        assert!(dir.path().join("keep.mp3").exists());
        assert!(!dir.path().join("orphan.mp3").exists());
    }
}
