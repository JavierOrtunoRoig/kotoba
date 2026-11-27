use serde::{Deserialize, Serialize};

/// Persisted settings. Stored as JSON in the `settings` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub interval_min_minutes: i64,
    pub interval_max_minutes: i64,
    pub max_per_day: i64,
    pub paused: bool,
    pub dnd_enabled: bool,
    pub dnd_start: String,
    pub dnd_end: String,
    /// Automatically play audio in the JP → ES direction.
    pub audio_autoplay: bool,
    /// `popup`, `widget` or `toast`.
    pub mode: String,
    /// Do not show challenges if a full-screen app or a presentation is running.
    pub suppress_fullscreen: bool,
    /// `random` or `alternate`.
    pub direction_mode: String,
    /// Include grammar/pattern notes (〜, +, ...).
    pub include_patterns: bool,
    pub launch_at_startup: bool,
    /// Target reviews per week (0 = no goal).
    pub weekly_goal: i64,
    /// Notify milestones such as completing the weekly goal or unlocking achievements.
    pub milestone_notifications: bool,
    /// Percentage of mastered words needed to complete a lesson (50-100).
    pub lesson_mastery_percent: i64,
    /// Automatically repair a missed day if a streak saver is available.
    pub streak_repair: bool,
    /// Visual theme: `auto`, `light` or `dark`.
    pub theme: String,
    /// UI scale in percent (90-125).
    pub font_scale: i64,
    /// UI language: `es`, `en` or `ja`.
    pub language: String,
    /// Maximum new words per day (0 = do not introduce new ones).
    pub new_per_day: i64,
    /// Percentage of new words when there are due reviews too (0 = reviews first).
    pub new_mix_percent: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval_min_minutes: 10,
            interval_max_minutes: 25,
            max_per_day: 40,
            paused: false,
            dnd_enabled: true,
            dnd_start: "22:00".to_string(),
            dnd_end: "08:00".to_string(),
            audio_autoplay: true,
            mode: "popup".to_string(),
            suppress_fullscreen: true,
            direction_mode: "random".to_string(),
            include_patterns: false,
            launch_at_startup: false,
            weekly_goal: 30,
            milestone_notifications: false,
            lesson_mastery_percent: 80,
            streak_repair: true,
            theme: "auto".to_string(),
            font_scale: 100,
            language: "es".to_string(),
            new_per_day: 10,
            new_mix_percent: 0,
        }
    }
}

impl Settings {
    pub fn sanitize(mut self) -> Self {
        self.interval_min_minutes = self.interval_min_minutes.clamp(1, 240);
        self.interval_max_minutes = self
            .interval_max_minutes
            .clamp(self.interval_min_minutes, 480);
        self.max_per_day = self.max_per_day.clamp(0, 500);
        self.weekly_goal = self.weekly_goal.clamp(0, 500);
        self.lesson_mastery_percent = self.lesson_mastery_percent.clamp(50, 100);
        self.font_scale = self.font_scale.clamp(90, 125);
        self.new_per_day = self.new_per_day.clamp(0, 200);
        self.new_mix_percent = self.new_mix_percent.clamp(0, 100);
        if !matches!(self.theme.as_str(), "auto" | "light" | "dark") {
            self.theme = "auto".to_string();
        }
        if !matches!(self.language.as_str(), "es" | "en" | "ja") {
            self.language = "es".to_string();
        }
        if !matches!(self.direction_mode.as_str(), "random" | "alternate") {
            self.direction_mode = "random".to_string();
        }
        if !matches!(self.mode.as_str(), "popup" | "widget" | "toast") {
            self.mode = "popup".to_string();
        }
        if !is_hhmm(&self.dnd_start) {
            self.dnd_start = "22:00".to_string();
        }
        if !is_hhmm(&self.dnd_end) {
            self.dnd_end = "08:00".to_string();
        }
        self
    }
}

fn is_hhmm(value: &str) -> bool {
    let mut parts = value.split(':');
    let (Some(h), Some(m), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    match (h.parse::<u32>(), m.parse::<u32>()) {
        (Ok(h), Ok(m)) => h < 24 && m < 60,
        _ => false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub kanji: Option<String>,
    pub reading: String,
    pub meaning: String,
    pub kanji_gloss: Option<String>,
    pub lesson: String,
    pub category: String,
    pub is_pattern: bool,
    pub irregular: bool,
    pub has_audio: bool,
}

impl Item {
    /// Japanese form shown in JP → ES.
    #[allow(dead_code)]
    pub fn japanese_display(&self) -> &str {
        self.kanji
            .as_deref()
            .filter(|k| !k.trim().is_empty())
            .unwrap_or(&self.reading)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub item: Item,
    /// `jp_to_es` or `es_to_jp`.
    pub direction: String,
    /// Due reviews (not counting new words).
    pub due_count: i64,
    /// New words available that have not been introduced.
    pub new_count: i64,
    /// Seconds until the next scheduled review (null if no seen words remain).
    pub next_due_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerResult {
    pub correct: bool,
    /// Accepted answers; the first one is the primary one.
    pub expected: Vec<String>,
    pub reading: String,
    pub meaning: String,
    pub kanji_gloss: Option<String>,
    pub has_audio: bool,
    /// Minutes until the next review of this item.
    pub next_interval_minutes: i64,
    /// XP earned in this review (0 if it failed).
    pub xp_gained: i64,
    /// Total accumulated XP.
    pub xp_total: i64,
    /// Current level.
    pub level: i64,
    /// Whether this review leveled up.
    pub leveled_up: bool,
    /// Achievements unlocked with this review.
    pub unlocked: Vec<Achievement>,
    /// Whether this review completed the weekly goal and granted a streak saver.
    pub saver_earned: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DayCount {
    pub date: String,
    pub total: i64,
    pub correct: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LessonStat {
    pub lesson: String,
    pub total: i64,
    pub reviewed: i64,
    pub new: i64,
    pub learning: i64,
    pub relearning: i64,
    pub young: i64,
    pub mastered: i64,
    pub reviews: i64,
    pub correct: i64,
    pub accuracy: f64,
    /// Percentage of mastered words over the lesson total.
    pub mastery_percent: f64,
    /// Whether the lesson reaches the configured mastery threshold.
    pub is_mastered: bool,
}

/// Achievement or badge. `progress` is clamped to `target`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Achievement {
    pub key: String,
    pub title: String,
    pub description: String,
    pub unlocked: bool,
    /// Unlock timestamp (Unix seconds).
    pub unlocked_at: Option<i64>,
    pub progress: i64,
    pub target: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DifficultItem {
    pub id: i64,
    pub japanese: String,
    pub meaning: String,
    pub attempts: i64,
    pub failures: i64,
    pub lapses: i64,
}

/// A review from the history, with the data of the item it belongs to.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewLogEntry {
    /// Review timestamp (Unix seconds).
    pub answered_at: i64,
    /// The same timestamp in local time, `YYYY-MM-DD HH:MM:SS`.
    pub answered_at_local: String,
    pub reading: String,
    pub meaning: String,
    /// `jp_to_es` or `es_to_jp`.
    pub direction: String,
    /// `true` if it was correct.
    pub result: bool,
    pub quality: i64,
    pub latency_ms: i64,
}

/// Page of the review history.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewHistoryPage {
    pub entries: Vec<ReviewLogEntry>,
    /// Reviews matching the filters, not only those on this page.
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub has_more: bool,
}

/// Filters for a history query.
#[derive(Debug, Clone, Default)]
pub struct ReviewHistoryQuery {
    pub page: i64,
    pub per_page: i64,
    /// `jp_to_es` or `es_to_jp`; any other value is ignored.
    pub direction: Option<String>,
    /// `Some(true)` correct, `Some(false)` wrong.
    pub result: Option<bool>,
    /// From this timestamp (inclusive).
    pub from_ts: Option<i64>,
    /// Up to this timestamp (exclusive); the UI sends midnight of the following day.
    pub to_ts: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub total_items: i64,
    pub total_reviews: i64,
    pub reviewed_today: i64,
    pub correct_today: i64,
    pub accuracy_today: f64,
    pub streak_days: i64,
    pub mastered: i64,
    /// Due reviews (not counting new words).
    pub due_now: i64,
    /// New words not yet introduced.
    pub new_available: i64,
    /// New words introduced today.
    pub new_today: i64,
    /// Seconds until the next scheduled review (null if no seen words remain).
    pub next_due_seconds: Option<i64>,
    /// Daily limit of new words.
    pub new_per_day: i64,
    pub activity: Vec<DayCount>,
    pub lessons: Vec<LessonStat>,
    pub difficult: Vec<DifficultItem>,
    /// Accumulated XP.
    pub xp_total: i64,
    /// Current level.
    pub level: i64,
    /// Name of the level tier (Semilla, Brote, ...).
    pub level_title: String,
    /// Progress 0-100 towards the next level.
    pub level_progress: f64,
    /// Absolute XP of the next level (null if it is the maximum).
    pub next_level_xp: Option<i64>,
    /// Best streak ever.
    pub streak_best: i64,
    /// Available streak savers.
    pub streak_savers: i64,
    /// Maximum stackable streak savers.
    pub streak_savers_max: i64,
    /// Streak days repaired with savers.
    pub streak_repairs: i64,
    /// Reviews in the current week.
    pub week_reviews: i64,
    /// Configured weekly goal (0 = no goal).
    pub weekly_goal: i64,
    /// Mastery threshold per lesson (50-100).
    pub lesson_mastery_percent: i64,
    /// Achievement catalog with current progress.
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bootstrap {
    pub item_count: i64,
    pub settings: Settings,
    pub stats: Stats,
    /// `true` in development builds. The UI disables what only works in release
    /// (for example, starting with Windows: a debug binary opens a console and loads the devUrl).
    pub dev_build: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportSummary {
    pub notes: i64,
    pub items_inserted: i64,
    pub items_merged: i64,
    pub sources: i64,
    pub audio_extracted: i64,
    pub audio_reused: i64,
    pub audio_missing: i64,
    pub patterns: i64,
    pub deck_name: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportProgress {
    pub phase: String,
    pub current: i64,
    pub total: i64,
}

/// Column/field indices that feed each kotoba datum.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMapping {
    pub reading: Option<usize>,
    pub kanji: Option<usize>,
    pub meaning: Option<usize>,
    pub gloss: Option<usize>,
    pub audio: Option<usize>,
    pub uid: Option<usize>,
    pub tags: Option<usize>,
    pub lesson: Option<usize>,
    pub category: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTypeMapping {
    pub mid: i64,
    pub mapping: FieldMapping,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ImportMapping {
    /// Mapping for the `.txt` file (positional columns).
    pub txt: Option<FieldMapping>,
    /// Mapping per Anki note type.
    pub note_types: Vec<NoteTypeMapping>,
    /// Media folder for the `.txt`; by default, the sibling with the same name.
    pub media_folder: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoteTypeInfo {
    pub mid: i64,
    pub name: String,
    pub fields: Vec<String>,
    pub count: i64,
    pub sample: Vec<Vec<String>>,
    pub suggested: FieldMapping,
}

/// Result of inspecting a deck before importing it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportInspection {
    /// `txt` or `apkg`.
    pub kind: String,
    pub deck_name: String,
    pub total_notes: i64,
    /// Columns of the `.txt` (empty for `.apkg`).
    pub columns: Vec<String>,
    pub sample: Vec<Vec<String>>,
    pub suggested: Option<FieldMapping>,
    pub note_types: Vec<NoteTypeInfo>,
    /// Detected sibling audio folder for a `.txt`.
    pub media_folder: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lesson_dominance_default_preserves_stored_threshold() {
        assert_eq!(Settings::default().lesson_mastery_percent, 80);
        let stored: Settings = serde_json::from_str(r#"{"lesson_mastery_percent":90}"#).unwrap();
        assert_eq!(stored.lesson_mastery_percent, 90);
    }

    #[test]
    fn sanitizes_invalid_mode_and_intervals() {
        let settings = Settings {
            mode: "nope".to_string(),
            direction_mode: "sideways".to_string(),
            interval_min_minutes: 60,
            interval_max_minutes: 5,
            max_per_day: 999,
            dnd_start: "99:99".to_string(),
            ..Default::default()
        }
        .sanitize();

        assert_eq!(settings.mode, "popup");
        assert_eq!(settings.direction_mode, "random");
        assert!(settings.interval_max_minutes >= settings.interval_min_minutes);
        assert_eq!(settings.max_per_day, 500);
        assert_eq!(settings.dnd_start, "22:00");
    }

    #[test]
    fn keeps_valid_modes() {
        for mode in ["popup", "widget", "toast"] {
            let settings = Settings {
                mode: mode.to_string(),
                ..Default::default()
            }
            .sanitize();
            assert_eq!(settings.mode, mode);
        }
    }

    #[test]
    fn clamps_weekly_goal() {
        let high = Settings {
            weekly_goal: 999,
            ..Default::default()
        }
        .sanitize();
        assert_eq!(high.weekly_goal, 500);

        let low = Settings {
            weekly_goal: -5,
            ..Default::default()
        }
        .sanitize();
        assert_eq!(low.weekly_goal, 0);
    }

    #[test]
    fn clamps_lesson_mastery_percent() {
        let high = Settings {
            lesson_mastery_percent: 200,
            ..Default::default()
        }
        .sanitize();
        assert_eq!(high.lesson_mastery_percent, 100);

        let low = Settings {
            lesson_mastery_percent: 10,
            ..Default::default()
        }
        .sanitize();
        assert_eq!(low.lesson_mastery_percent, 50);
    }

    #[test]
    fn sanitizes_appearance_and_language() {
        let settings = Settings {
            theme: "neon".to_string(),
            font_scale: 400,
            language: "fr".to_string(),
            new_per_day: 500,
            new_mix_percent: -10,
            ..Default::default()
        }
        .sanitize();
        assert_eq!(settings.theme, "auto");
        assert_eq!(settings.font_scale, 125);
        assert_eq!(settings.language, "es");
        assert_eq!(settings.new_per_day, 200);
        assert_eq!(settings.new_mix_percent, 0);

        let settings = Settings {
            font_scale: 10,
            ..Default::default()
        }
        .sanitize();
        assert_eq!(settings.font_scale, 90);
    }
}
