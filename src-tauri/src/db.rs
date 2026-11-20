use crate::models::{Achievement, Item, Settings, Stats};
use anyhow::Result;
use chrono::{Local, NaiveDate, TimeZone};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Intervalos de aprendizaje antes de entrar en el ciclo SM-2.
const LEARNING_INTERVAL_MINUTES: i64 = 10;
const FIRST_REVIEW_INTERVAL_MINUTES: i64 = 1440;
pub const MAX_QUALITY: u8 = 5;

/// Intervalo máximo: 10 años, para evitar intervalos absurdos y desbordamientos.
const MAX_INTERVAL_MINUTES: i64 = 3650 * 1440;

/// Salvavidas de racha acumulables.
pub const MAX_STREAK_SAVERS: i64 = 2;

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
            (reps + 1, interval.min(MAX_INTERVAL_MINUTES))
        }
    };

    (reps, interval, ease)
}

/// XP por respuesta correcta según la calidad SM-2. Un fallo no da XP.
pub fn xp_for_quality(quality: u8) -> i64 {
    match quality {
        5 => 3,
        4 => 2,
        3 => 1,
        _ => 0,
    }
}

/// Umbrales absolutos de XP para cada nivel.
const LEVEL_THRESHOLDS: [i64; 12] = [
    0, 80, 200, 380, 650, 1000, 1450, 2000, 2700, 3500, 4500, 5800,
];

/// Nombre del tramo de nivel. Da contexto sin añadir mecánicas nuevas.
pub fn level_title(level: i64) -> &'static str {
    match level {
        1..=2 => "Semilla",
        3..=4 => "Brote",
        5..=6 => "Raíz",
        7..=8 => "Rama",
        9..=10 => "Árbol",
        _ => "Bosque",
    }
}

pub fn level_for_xp(xp: i64) -> i64 {
    LEVEL_THRESHOLDS
        .iter()
        .rposition(|threshold| xp >= *threshold)
        .map(|index| index as i64 + 1)
        .unwrap_or(1)
}

/// Devuelve `(nivel, progreso 0-100, xp del siguiente nivel)`.
pub fn level_progress(xp: i64) -> (i64, f64, Option<i64>) {
    let level = level_for_xp(xp);
    let current = LEVEL_THRESHOLDS[(level - 1) as usize];
    let next = LEVEL_THRESHOLDS.get(level as usize).copied();
    let progress = match next {
        Some(next) => {
            let span = (next - current).max(1) as f64;
            ((xp - current) as f64 / span * 100.0).clamp(0.0, 100.0)
        }
        None => 100.0,
    };
    (level, progress, next)
}

pub struct ReviewOutcome {
    pub next_interval_minutes: i64,
    pub xp_gained: i64,
    pub xp_total: i64,
    pub level: i64,
    pub leveled_up: bool,
}

/// Métrica que alimenta cada logro.
#[derive(Clone, Copy)]
enum AchievementMetric {
    Reviews,
    Mastered,
    Streak,
    LessonsMastered,
    Level,
    WeeklyGoals,
}

struct AchievementDef {
    key: &'static str,
    title: &'static str,
    description: &'static str,
    metric: AchievementMetric,
    target: i64,
}

/// Catálogo de logros. El orden es el orden de presentación.
const ACHIEVEMENT_CATALOG: [AchievementDef; 16] = [
    AchievementDef {
        key: "first_step",
        title: "Primer paso",
        description: "Completa tu primer repaso.",
        metric: AchievementMetric::Reviews,
        target: 1,
    },
    AchievementDef {
        key: "reviews_25",
        title: "Calentando",
        description: "Completa 25 repasos.",
        metric: AchievementMetric::Reviews,
        target: 25,
    },
    AchievementDef {
        key: "reviews_100",
        title: "Cien repasos",
        description: "Completa 100 repasos.",
        metric: AchievementMetric::Reviews,
        target: 100,
    },
    AchievementDef {
        key: "reviews_250",
        title: "Constancia",
        description: "Completa 250 repasos.",
        metric: AchievementMetric::Reviews,
        target: 250,
    },
    AchievementDef {
        key: "reviews_500",
        title: "Medio millar",
        description: "Completa 500 repasos.",
        metric: AchievementMetric::Reviews,
        target: 500,
    },
    AchievementDef {
        key: "mastered_1",
        title: "Primera semilla",
        description: "Domina una palabra con un intervalo de 7 días o más.",
        metric: AchievementMetric::Mastered,
        target: 1,
    },
    AchievementDef {
        key: "mastered_10",
        title: "Diez semillas",
        description: "Domina 10 palabras.",
        metric: AchievementMetric::Mastered,
        target: 10,
    },
    AchievementDef {
        key: "mastered_25",
        title: "Veinticinco semillas",
        description: "Domina 25 palabras.",
        metric: AchievementMetric::Mastered,
        target: 25,
    },
    AchievementDef {
        key: "mastered_50",
        title: "Cincuenta semillas",
        description: "Domina 50 palabras.",
        metric: AchievementMetric::Mastered,
        target: 50,
    },
    AchievementDef {
        key: "streak_3",
        title: "Tres días",
        description: "Repasa 3 días seguidos.",
        metric: AchievementMetric::Streak,
        target: 3,
    },
    AchievementDef {
        key: "streak_7",
        title: "Una semana",
        description: "Repasa 7 días seguidos.",
        metric: AchievementMetric::Streak,
        target: 7,
    },
    AchievementDef {
        key: "streak_30",
        title: "Un mes",
        description: "Repasa 30 días seguidos.",
        metric: AchievementMetric::Streak,
        target: 30,
    },
    AchievementDef {
        key: "lesson_1",
        title: "Lección dominada",
        description: "Alcanza el umbral de maestría en una lección.",
        metric: AchievementMetric::LessonsMastered,
        target: 1,
    },
    AchievementDef {
        key: "lesson_5",
        title: "Cinco lecciones",
        description: "Alcanza el umbral de maestría en cinco lecciones.",
        metric: AchievementMetric::LessonsMastered,
        target: 5,
    },
    AchievementDef {
        key: "level_5",
        title: "Nivel 5",
        description: "Alcanza el nivel 5.",
        metric: AchievementMetric::Level,
        target: 5,
    },
    AchievementDef {
        key: "weekly_goal_1",
        title: "Objetivo cumplido",
        description: "Completa un objetivo semanal.",
        metric: AchievementMetric::WeeklyGoals,
        target: 1,
    },
];

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

        CREATE TABLE IF NOT EXISTS profile (
            id                INTEGER PRIMARY KEY CHECK (id = 1),
            xp_total          INTEGER NOT NULL DEFAULT 0,
            level             INTEGER NOT NULL DEFAULT 1,
            streak_best       INTEGER NOT NULL DEFAULT 0,
            weekly_goals_done INTEGER NOT NULL DEFAULT 0,
            streak_savers     INTEGER NOT NULL DEFAULT 0
        );

        INSERT OR IGNORE INTO profile (id, xp_total, level, streak_best)
        VALUES (1, 0, 1, 0);

        CREATE TABLE IF NOT EXISTS achievements (
            key         TEXT PRIMARY KEY,
            unlocked_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS streak_repairs (
            day        TEXT PRIMARY KEY,
            created_at INTEGER NOT NULL
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
    if version < 2 {
        let current = streak_days(conn)?;
        conn.execute(
            "UPDATE profile SET streak_best = MAX(streak_best, ?1) WHERE id = 1",
            [current],
        )?;
        conn.pragma_update(None, "user_version", 2)?;
    }
    if version < 3 {
        ensure_column(
            conn,
            "profile",
            "weekly_goals_done",
            "INTEGER NOT NULL DEFAULT 0",
        )?;
        conn.pragma_update(None, "user_version", 3)?;
    }
    if version < 4 {
        ensure_column(conn, "profile", "streak_savers", "INTEGER NOT NULL DEFAULT 0")?;
        conn.pragma_update(None, "user_version", 4)?;
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

/// Lunes a las 00:00 local.
pub fn week_start_ts() -> i64 {
    use chrono::Datelike;

    let today = Local::now().date_naive();
    let days_from_monday = today.weekday().num_days_from_monday() as i64;
    let monday = today - chrono::Duration::days(days_from_monday);
    monday
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| Local.from_local_datetime(&dt).single())
        .map(|dt| dt.timestamp())
        .unwrap_or_else(today_start_ts)
}

pub fn reviews_since(conn: &Connection, since: i64) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM review_log WHERE answered_at >= ?1",
        [since],
        |row| row.get(0),
    )?)
}

pub fn reviews_this_week(conn: &Connection) -> Result<i64> {
    reviews_since(conn, week_start_ts())
}

pub fn profile(conn: &Connection) -> Result<(i64, i64, i64)> {
    Ok(conn.query_row(
        "SELECT xp_total, level, streak_best FROM profile WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?)
}

pub fn weekly_goals_done(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT weekly_goals_done FROM profile WHERE id = 1",
        [],
        |row| row.get(0),
    )?)
}

pub fn streak_savers(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT streak_savers FROM profile WHERE id = 1",
        [],
        |row| row.get(0),
    )?)
}

/// Añade un salvavidas si no se ha alcanzado el máximo. Devuelve si se añadió.
pub fn add_streak_saver(conn: &Connection) -> Result<bool> {
    let current = streak_savers(conn)?;
    if current >= MAX_STREAK_SAVERS {
        return Ok(false);
    }
    conn.execute(
        "UPDATE profile SET streak_savers = streak_savers + 1 WHERE id = 1",
        [],
    )?;
    Ok(true)
}

pub fn streak_repairs_count(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM streak_repairs", [], |row| {
        row.get(0)
    })?)
}

/// Días con actividad: repasos reales más días reparados con salvavidas.
fn active_days(conn: &Connection) -> Result<HashSet<NaiveDate>> {
    let mut stmt = conn.prepare(
        "SELECT day FROM (
             SELECT DISTINCT date(answered_at, 'unixepoch', 'localtime') AS day
             FROM review_log WHERE answered_at IS NOT NULL
             UNION
             SELECT day FROM streak_repairs
         )",
    )?;
    let days = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|row| row.ok())
        .filter_map(|value| NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok())
        .collect();
    Ok(days)
}

fn day_ts(date: NaiveDate) -> Option<i64> {
    date.and_hms_opt(12, 0, 0)
        .and_then(|dt| Local.from_local_datetime(&dt).single())
        .map(|dt| dt.timestamp())
}

/// Repara como mucho el hueco más reciente de la racha usando un salvavidas.
///
/// Solo actúa si hay un día perdido entre la racha actual y actividad anterior.
/// Devuelve el día reparado.
pub fn maintain_streak(conn: &Connection, enabled: bool) -> Result<Option<String>> {
    if !enabled || streak_savers(conn)? <= 0 {
        return Ok(None);
    }

    let days = active_days(conn)?;
    if days.is_empty() {
        return Ok(None);
    }

    let today = Local::now().date_naive();
    let mut cursor = if days.contains(&today) {
        today
    } else {
        match today.pred_opt() {
            Some(yesterday) if days.contains(&yesterday) => yesterday,
            _ => return Ok(None),
        }
    };

    while days.contains(&cursor) {
        match cursor.pred_opt() {
            Some(previous) => cursor = previous,
            None => return Ok(None),
        }
    }

    let has_older_activity = cursor
        .pred_opt()
        .map(|date| days.contains(&date))
        .unwrap_or(false);
    if !has_older_activity {
        return Ok(None);
    }

    let Some(repaired_at) = day_ts(cursor) else {
        return Ok(None);
    };
    let day = cursor.format("%Y-%m-%d").to_string();
    conn.execute(
        "INSERT OR IGNORE INTO streak_repairs (day, created_at) VALUES (?1, ?2)",
        params![day, repaired_at],
    )?;
    conn.execute(
        "UPDATE profile SET streak_savers = MAX(streak_savers - 1, 0) WHERE id = 1",
        [],
    )?;
    let current = streak_days(conn)?;
    conn.execute(
        "UPDATE profile SET streak_best = MAX(streak_best, ?1) WHERE id = 1",
        [current],
    )?;
    Ok(Some(day))
}

/// Suma un objetivo semanal completado. Se llama al cruzar el umbral.
pub fn bump_weekly_goals_done(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE profile SET weekly_goals_done = weekly_goals_done + 1 WHERE id = 1",
        [],
    )?;
    Ok(())
}

/// Catálogo completo con el progreso actual y si ya está desbloqueado.
pub fn achievements(conn: &Connection, mastery_percent: i64) -> Result<Vec<Achievement>> {
    let (xp_total, _stored_level, stored_best) = profile(conn).unwrap_or((0, 1, 0));
    let level = level_for_xp(xp_total);
    let total_reviews: i64 =
        conn.query_row("SELECT COUNT(*) FROM review_log", [], |row| row.get(0))?;
    let mastered: i64 = conn.query_row(
        "SELECT COUNT(*) FROM srs_state WHERE interval_minutes >= 10080",
        [],
        |row| row.get(0),
    )?;
    let streak_best = stored_best.max(streak_days(conn)?);
    let lessons_mastered = lessons(conn, mastery_percent)?
        .iter()
        .filter(|lesson| lesson.lesson != "Sin lección" && lesson.is_mastered)
        .count() as i64;
    let weekly_goals = weekly_goals_done(conn)?;

    let mut unlocked_stmt = conn.prepare("SELECT key, unlocked_at FROM achievements")?;
    let unlocked: HashMap<String, i64> = unlocked_stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?
        .filter_map(|row| row.ok())
        .collect();

    let mut out = Vec::with_capacity(ACHIEVEMENT_CATALOG.len());
    for def in ACHIEVEMENT_CATALOG.iter() {
        let value = match def.metric {
            AchievementMetric::Reviews => total_reviews,
            AchievementMetric::Mastered => mastered,
            AchievementMetric::Streak => streak_best,
            AchievementMetric::LessonsMastered => lessons_mastered,
            AchievementMetric::Level => level,
            AchievementMetric::WeeklyGoals => weekly_goals,
        };
        let unlocked_at = unlocked.get(def.key).copied();
        out.push(Achievement {
            key: def.key.to_string(),
            title: def.title.to_string(),
            description: def.description.to_string(),
            unlocked: unlocked_at.is_some(),
            unlocked_at,
            progress: value.clamp(0, def.target),
            target: def.target,
        });
    }
    Ok(out)
}

/// Desbloquea los logros que ya cumplen su objetivo y devuelve solo los nuevos.
pub fn sync_achievements(conn: &Connection, mastery_percent: i64) -> Result<Vec<Achievement>> {
    let now = now_ts();
    let mut newly = Vec::new();
    for mut achievement in achievements(conn, mastery_percent)? {
        if !achievement.unlocked && achievement.progress >= achievement.target {
            conn.execute(
                "INSERT OR IGNORE INTO achievements (key, unlocked_at) VALUES (?1, ?2)",
                params![achievement.key, now],
            )?;
            achievement.unlocked = true;
            achievement.unlocked_at = Some(now);
            newly.push(achievement);
        }
    }
    Ok(newly)
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
    reviews_since(conn, today_start_ts())
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
) -> Result<ReviewOutcome> {
    let now = now_ts();
    let correct = quality >= 3;
    let xp_gained = xp_for_quality(quality);
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

    let (old_xp, old_level, old_best): (i64, i64, i64) = tx
        .query_row(
            "SELECT xp_total, level, streak_best FROM profile WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .unwrap_or((0, 1, 0));

    let xp_total = old_xp + xp_gained;
    let level = level_for_xp(xp_total);
    let current_streak = streak_days(&tx)?;
    let streak_best = old_best.max(current_streak);

    tx.execute(
        "UPDATE profile SET xp_total = ?1, level = ?2, streak_best = ?3 WHERE id = 1",
        params![xp_total, level, streak_best],
    )?;

    tx.commit()?;
    Ok(ReviewOutcome {
        next_interval_minutes: interval,
        xp_gained,
        xp_total,
        level,
        leveled_up: level > old_level,
    })
}

pub fn stats(
    conn: &Connection,
    include_patterns: bool,
    weekly_goal: i64,
    lesson_mastery_percent: i64,
) -> Result<Stats> {
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

    let (xp_total, _stored_level, stored_best) = profile(conn).unwrap_or((0, 1, 0));
    let (level, level_progress_value, next_level_xp) = level_progress(xp_total);
    let current_streak = streak_days(conn)?;
    let streak_best = stored_best.max(current_streak);
    let week_reviews = reviews_this_week(conn)?;
    let savers = streak_savers(conn)?;
    let repairs = streak_repairs_count(conn)?;

    Ok(Stats {
        total_items,
        total_reviews,
        reviewed_today,
        correct_today,
        accuracy_today,
        streak_days: current_streak,
        mastered,
        due_now,
        activity: activity(conn)?,
        lessons: lessons(conn, lesson_mastery_percent)?,
        difficult: difficult(conn)?,
        xp_total,
        level,
        level_title: level_title(level).to_string(),
        level_progress: level_progress_value,
        next_level_xp,
        streak_best,
        streak_savers: savers,
        streak_savers_max: MAX_STREAK_SAVERS,
        streak_repairs: repairs,
        week_reviews,
        weekly_goal,
        lesson_mastery_percent,
        achievements: achievements(conn, lesson_mastery_percent)?,
    })
}

fn lessons(conn: &Connection, mastery_percent: i64) -> Result<Vec<crate::models::LessonStat>> {
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
        let mastery = if total > 0 {
            mastered as f64 / total as f64 * 100.0
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
            mastery_percent: mastery,
            is_mastered: total > 0 && mastery >= mastery_percent as f64,
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
    let days = active_days(conn)?;
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
    conn.execute_batch(
        "DELETE FROM review_log;
         DELETE FROM srs_state;
         DELETE FROM achievements;
         DELETE FROM streak_repairs;
         UPDATE profile
         SET xp_total = 0, level = 1, streak_best = 0, weekly_goals_done = 0,
             streak_savers = 0
         WHERE id = 1;",
    )?;
    Ok(())
}

pub fn clear_library(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM review_log;
         DELETE FROM srs_state;
         DELETE FROM item_sources;
         DELETE FROM items;
         DELETE FROM achievements;
         DELETE FROM streak_repairs;
         UPDATE profile
         SET xp_total = 0, level = 1, streak_best = 0, weekly_goals_done = 0,
             streak_savers = 0
         WHERE id = 1;",
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
    fn sm2_caps_interval() {
        let (_, interval, _) = sm2_next(5, 40, MAX_INTERVAL_MINUTES, 2.5);
        assert_eq!(interval, MAX_INTERVAL_MINUTES);
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

    #[test]
    fn xp_only_for_correct_quality() {
        assert_eq!(xp_for_quality(5), 3);
        assert_eq!(xp_for_quality(4), 2);
        assert_eq!(xp_for_quality(3), 1);
        assert_eq!(xp_for_quality(2), 0);
        assert_eq!(xp_for_quality(1), 0);
    }

    #[test]
    fn levels_scale_with_xp() {
        assert_eq!(level_for_xp(0), 1);
        assert_eq!(level_for_xp(79), 1);
        assert_eq!(level_for_xp(80), 2);
        assert_eq!(level_for_xp(199), 2);
        assert_eq!(level_for_xp(200), 3);
        assert_eq!(level_for_xp(5800), 12);
        assert_eq!(level_for_xp(99999), 12);

        assert_eq!(level_title(1), "Semilla");
        assert_eq!(level_title(4), "Brote");
        assert_eq!(level_title(5), "Raíz");
        assert_eq!(level_title(7), "Rama");
        assert_eq!(level_title(9), "Árbol");
        assert_eq!(level_title(11), "Bosque");
    }

    #[test]
    fn records_xp_and_levels_up() {
        let mut conn = Connection::open_in_memory().expect("db");
        migrate(&mut conn).expect("migrate");
        conn.execute(
            "INSERT INTO items
             (source_uid, reading, meaning, meaning_plain, dedup_key, created_at)
             VALUES ('u1', 'a', 'b', 'b', 'a|b', 0)",
            [],
        )
        .expect("item");

        let outcome = record_review(&mut conn, 1, 5, "es_to_jp", 1000).expect("review");
        assert_eq!(outcome.xp_gained, 3);
        assert_eq!(outcome.xp_total, 3);
        assert_eq!(outcome.level, 1);
        assert!(!outcome.leveled_up);

        let mut last = outcome;
        for _ in 0..26 {
            last = record_review(&mut conn, 1, 5, "es_to_jp", 1000).expect("review");
        }

        assert_eq!(last.xp_total, 81);
        assert_eq!(last.level, 2);
        assert!(last.leveled_up);

        let (xp, level, best) = profile(&conn).expect("profile");
        assert_eq!(xp, 81);
        assert_eq!(level, 2);
        assert_eq!(best, 1);
    }

    #[test]
    fn caps_streak_savers() {
        let mut conn = Connection::open_in_memory().expect("db");
        migrate(&mut conn).expect("migrate");

        assert!(add_streak_saver(&conn).expect("saver"));
        assert!(add_streak_saver(&conn).expect("saver"));
        assert!(!add_streak_saver(&conn).expect("saver"));
        assert_eq!(streak_savers(&conn).expect("count"), MAX_STREAK_SAVERS);
    }

    #[test]
    fn repairs_a_single_streak_gap() {
        use chrono::Duration;

        let mut conn = Connection::open_in_memory().expect("db");
        migrate(&mut conn).expect("migrate");
        conn.execute(
            "INSERT INTO items
             (source_uid, reading, meaning, meaning_plain, dedup_key, created_at)
             VALUES ('u1', 'a', 'b', 'b', 'a|b', 0)",
            [],
        )
        .expect("item");

        let today = Local::now().date_naive();
        for offset in [0, 2] {
            let date = today - Duration::days(offset);
            let answered_at = date
                .and_hms_opt(12, 0, 0)
                .and_then(|dt| Local.from_local_datetime(&dt).single())
                .expect("timestamp")
                .timestamp();
            conn.execute(
                "INSERT INTO review_log
                 (item_id, shown_at, answered_at, result, quality, latency_ms, direction)
                 VALUES (1, ?1, ?1, 1, 5, 1000, 'jp_to_es')",
                [answered_at],
            )
            .expect("review");
        }

        add_streak_saver(&conn).expect("saver");
        assert!(maintain_streak(&conn, false).expect("disabled").is_none());
        assert_eq!(streak_savers(&conn).expect("count"), 1);

        let repaired = maintain_streak(&conn, true).expect("repair");
        let yesterday = (today - Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(repaired, Some(yesterday));
        assert_eq!(streak_days(&conn).expect("streak"), 3);
        assert_eq!(streak_savers(&conn).expect("count"), 0);
        assert_eq!(streak_repairs_count(&conn).expect("repairs"), 1);
        assert!(maintain_streak(&conn, true).expect("repair").is_none());
    }

    #[test]
    fn unlocks_achievements_once() {
        let mut conn = Connection::open_in_memory().expect("db");
        migrate(&mut conn).expect("migrate");
        conn.execute(
            "INSERT INTO items
             (source_uid, reading, meaning, meaning_plain, dedup_key, created_at)
             VALUES ('u1', 'a', 'b', 'b', 'a|b', 0)",
            [],
        )
        .expect("item");

        assert!(sync_achievements(&conn, 90).expect("sync").is_empty());

        record_review(&mut conn, 1, 5, "es_to_jp", 1000).expect("review");
        let newly = sync_achievements(&conn, 90).expect("sync");
        assert_eq!(newly.len(), 1);
        assert_eq!(newly[0].key, "first_step");
        assert!(newly[0].unlocked_at.is_some());

        assert!(sync_achievements(&conn, 90).expect("sync").is_empty());

        let catalog = achievements(&conn, 90).expect("achievements");
        let first = catalog
            .iter()
            .find(|achievement| achievement.key == "first_step")
            .expect("first_step");
        assert!(first.unlocked);
        assert_eq!(first.progress, 1);
    }

    #[test]
    fn migrates_v2_profile_in_place() {
        let mut conn = Connection::open_in_memory().expect("db");
        conn.execute_batch(
            "CREATE TABLE profile (
                 id          INTEGER PRIMARY KEY CHECK (id = 1),
                 xp_total    INTEGER NOT NULL DEFAULT 0,
                 level       INTEGER NOT NULL DEFAULT 1,
                 streak_best INTEGER NOT NULL DEFAULT 0
             );
             INSERT INTO profile (id, xp_total, level, streak_best) VALUES (1, 42, 3, 2);
             PRAGMA user_version = 2;",
        )
        .expect("old schema");

        migrate(&mut conn).expect("migrate");
        assert_eq!(profile(&conn).expect("profile"), (42, 3, 2));
        assert_eq!(weekly_goals_done(&conn).expect("weekly goals"), 0);
        assert_eq!(streak_savers(&conn).expect("savers"), 0);
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("version");
        assert_eq!(version, 4);
    }

    #[test]
    fn lesson_mastery_uses_threshold() {
        let mut conn = Connection::open_in_memory().expect("db");
        migrate(&mut conn).expect("migrate");
        for index in 0..10 {
            conn.execute(
                "INSERT INTO items
                 (source_uid, reading, meaning, meaning_plain, lesson, dedup_key, created_at)
                 VALUES (?1, 'a', 'b', 'b', 'L1', ?2, 0)",
                params![format!("u{index}"), format!("a|b|{index}")],
            )
            .expect("item");
            if index < 9 {
                conn.execute(
                    "INSERT INTO srs_state (item_id, interval_minutes, ease, due_at, reps)
                     VALUES (?1, 10080, 2.5, 0, 3)",
                    [index + 1],
                )
                .expect("srs");
            }
        }

        let strict = lessons(&conn, 95).expect("lessons");
        assert_eq!(strict.len(), 1);
        assert!(!strict[0].is_mastered);
        assert!((strict[0].mastery_percent - 90.0).abs() < 1e-9);

        let relaxed = lessons(&conn, 90).expect("lessons");
        assert!(relaxed[0].is_mastered);
    }
}
