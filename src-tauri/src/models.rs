use serde::{Deserialize, Serialize};

/// Configuración persistida. Se guarda como JSON en la tabla `settings`.
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
    /// Reproducir el audio automáticamente en la dirección JP → ES.
    pub audio_autoplay: bool,
    /// `popup`, `widget` o `toast`.
    pub mode: String,
    /// No mostrar retos si hay una app a pantalla completa o una presentación.
    pub suppress_fullscreen: bool,
    /// `random` o `alternate`.
    pub direction_mode: String,
    /// Incluir notas de gramática/plantillas (〜, +, ...).
    pub include_patterns: bool,
    pub launch_at_startup: bool,
    /// Repasos objetivo por semana (0 = sin objetivo).
    pub weekly_goal: i64,
    /// Notificar hitos como completar el objetivo semanal o desbloquear logros.
    pub milestone_notifications: bool,
    /// Porcentaje de palabras dominadas para completar una lección (50-100).
    pub lesson_mastery_percent: i64,
    /// Reparar automáticamente un día perdido si hay un salvavidas disponible.
    pub streak_repair: bool,
    /// Tema visual: `auto`, `light` o `dark`.
    pub theme: String,
    /// Escala de la interfaz en porcentaje (90-125).
    pub font_scale: i64,
    /// Idioma de la interfaz: `es`, `en` o `ja`.
    pub language: String,
    /// Palabras nuevas máximas por día (0 = no introducir nuevas).
    pub new_per_day: i64,
    /// Porcentaje de nuevas cuando también hay repasos vencidos (0 = repasos primero).
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
            lesson_mastery_percent: 90,
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
    /// Forma japonesa que se muestra en JP → ES.
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
    /// `jp_to_es` o `es_to_jp`.
    pub direction: String,
    /// Repasos vencidos (sin contar palabras nuevas).
    pub due_count: i64,
    /// Palabras nuevas disponibles sin introducir.
    pub new_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerResult {
    pub correct: bool,
    /// Respuestas aceptadas, la primera es la principal.
    pub expected: Vec<String>,
    pub reading: String,
    pub meaning: String,
    pub kanji_gloss: Option<String>,
    pub has_audio: bool,
    /// Minutos hasta el próximo repaso de este ítem.
    pub next_interval_minutes: i64,
    /// XP ganado en este repaso (0 si falló).
    pub xp_gained: i64,
    /// XP total acumulado.
    pub xp_total: i64,
    /// Nivel actual.
    pub level: i64,
    /// Si este repaso ha subido de nivel.
    pub leveled_up: bool,
    /// Logros desbloqueados con este repaso.
    pub unlocked: Vec<Achievement>,
    /// Si este repaso ha completado el objetivo semanal y ha dado un salvavidas.
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
    pub mastered: i64,
    pub reviews: i64,
    pub correct: i64,
    pub accuracy: f64,
    /// Porcentaje de palabras dominadas sobre el total de la lección.
    pub mastery_percent: f64,
    /// Si la lección alcanza el umbral de maestría configurado.
    pub is_mastered: bool,
}

/// Logro o insignia. `progress` se recorta en `target`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Achievement {
    pub key: String,
    pub title: String,
    pub description: String,
    pub unlocked: bool,
    /// Marca temporal de desbloqueo (segundos Unix).
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub total_items: i64,
    pub total_reviews: i64,
    pub reviewed_today: i64,
    pub correct_today: i64,
    pub accuracy_today: f64,
    pub streak_days: i64,
    pub mastered: i64,
    /// Repasos vencidos (sin contar palabras nuevas).
    pub due_now: i64,
    /// Palabras nuevas sin introducir.
    pub new_available: i64,
    /// Palabras nuevas introducidas hoy.
    pub new_today: i64,
    /// Límite diario de palabras nuevas.
    pub new_per_day: i64,
    pub activity: Vec<DayCount>,
    pub lessons: Vec<LessonStat>,
    pub difficult: Vec<DifficultItem>,
    /// XP acumulado.
    pub xp_total: i64,
    /// Nivel actual.
    pub level: i64,
    /// Nombre del tramo del nivel (Semilla, Brote, ...).
    pub level_title: String,
    /// Progreso 0-100 hacia el siguiente nivel.
    pub level_progress: f64,
    /// XP absoluto del siguiente nivel (null si es el máximo).
    pub next_level_xp: Option<i64>,
    /// Mejor racha histórica.
    pub streak_best: i64,
    /// Salvavidas de racha disponibles.
    pub streak_savers: i64,
    /// Máximo de salvavidas acumulables.
    pub streak_savers_max: i64,
    /// Días de racha reparados con salvavidas.
    pub streak_repairs: i64,
    /// Repasos de la semana actual.
    pub week_reviews: i64,
    /// Objetivo semanal configurado (0 = sin objetivo).
    pub weekly_goal: i64,
    /// Umbral de maestría por lección (50-100).
    pub lesson_mastery_percent: i64,
    /// Catálogo de logros con su progreso actual.
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bootstrap {
    pub item_count: i64,
    pub settings: Settings,
    pub stats: Stats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportSummary {
    pub notes: i64,
    pub items_inserted: i64,
    pub items_merged: i64,
    pub sources: i64,
    pub audio_extracted: i64,
    pub audio_reused: i64,
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

/// Índices de columna/campo que alimentan cada dato de kotoba.
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
pub struct ImportMapping {
    /// Mapeo del archivo `.txt` (columnas posicionales).
    pub txt: Option<FieldMapping>,
    /// Mapeo por tipo de nota de Anki.
    pub note_types: Vec<NoteTypeMapping>,
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

/// Resultado de inspeccionar un mazo antes de importarlo.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportInspection {
    /// `txt` o `apkg`.
    pub kind: String,
    pub deck_name: String,
    pub total_notes: i64,
    /// Columnas del `.txt` (vacío en `.apkg`).
    pub columns: Vec<String>,
    pub sample: Vec<Vec<String>>,
    pub suggested: Option<FieldMapping>,
    pub note_types: Vec<NoteTypeInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
