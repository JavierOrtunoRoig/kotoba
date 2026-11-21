//! Traducciones del backend para la bandeja y las notificaciones nativas.

fn lang(language: &str) -> &str {
    match language {
        "en" => "en",
        "ja" => "ja",
        _ => "es",
    }
}

pub fn practice_now(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Practice now",
        "ja" => "今すぐ練習",
        _ => "Practicar ahora",
    }
}

pub fn open_app(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Open kotoba",
        "ja" => "kotoba を開く",
        _ => "Abrir kotoba",
    }
}

pub fn pause(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Pause",
        "ja" => "一時停止",
        _ => "Pausar",
    }
}

pub fn resume(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Resume",
        "ja" => "再開",
        _ => "Reanudar",
    }
}

pub fn quit(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Quit",
        "ja" => "終了",
        _ => "Salir",
    }
}

pub fn mode(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Mode",
        "ja" => "表示モード",
        _ => "Modo",
    }
}

pub fn tooltip(language: &str) -> &'static str {
    match lang(language) {
        "en" => "kotoba — passive Japanese review",
        "ja" => "kotoba — 受動的な日本語復習",
        _ => "kotoba — repaso pasivo de japonés",
    }
}

pub fn toast_body(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Tap to review a word",
        "ja" => "タップして単語を復習",
        _ => "Toca para repasar una palabra",
    }
}

pub fn weekly_title(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Weekly goal completed",
        "ja" => "週間目標を達成しました",
        _ => "Objetivo semanal completado",
    }
}

pub fn weekly_body(language: &str, reviews: i64, saver: bool) -> String {
    let mut body = match lang(language) {
        "en" => format!("{reviews} reviews this week. Well done!"),
        "ja" => format!("今週は{reviews}回の復習。よくできました！"),
        _ => format!("{reviews} repasos esta semana. ¡Bien hecho!"),
    };
    if saver {
        body.push_str(match lang(language) {
            "en" => " You earned a streak saver.",
            "ja" => " ストリーク保護を1つ獲得しました。",
            _ => " Has ganado un salvavidas de racha.",
        });
    }
    body
}

pub fn achievement_title(language: &str) -> &'static str {
    match lang(language) {
        "en" => "Achievement unlocked",
        "ja" => "実績を解除しました",
        _ => "Logro desbloqueado",
    }
}

pub fn achievement_body_one(language: &str, title: &str) -> String {
    match lang(language) {
        "en" => format!("You unlocked “{title}”"),
        "ja" => format!("「{title}」を解除しました"),
        _ => format!("Has desbloqueado «{title}»"),
    }
}

pub fn achievement_body_many(language: &str, count: usize) -> String {
    match lang(language) {
        "en" => format!("You unlocked {count} achievements"),
        "ja" => format!("実績を{count}個解除しました"),
        _ => format!("Has desbloqueado {count} logros"),
    }
}

/// Título localizado de un logro a partir de su clave.
pub fn achievement_name(language: &str, key: &str) -> String {
    let language = lang(language);
    let name = match (language, key) {
        ("en", "first_step") => "First step",
        ("en", "reviews_25") => "Warming up",
        ("en", "reviews_100") => "One hundred reviews",
        ("en", "reviews_250") => "Consistency",
        ("en", "reviews_500") => "Half a thousand",
        ("en", "mastered_1") => "First seed",
        ("en", "mastered_10") => "Ten seeds",
        ("en", "mastered_25") => "Twenty-five seeds",
        ("en", "mastered_50") => "Fifty seeds",
        ("en", "streak_3") => "Three days",
        ("en", "streak_7") => "One week",
        ("en", "streak_30") => "One month",
        ("en", "lesson_1") => "Lesson mastered",
        ("en", "lesson_5") => "Five lessons",
        ("en", "level_5") => "Level 5",
        ("en", "weekly_goal_1") => "Goal completed",
        ("ja", "first_step") => "最初の一歩",
        ("ja", "reviews_25") => "ウォームアップ",
        ("ja", "reviews_100") => "100回の復習",
        ("ja", "reviews_250") => "継続は力",
        ("ja", "reviews_500") => "500回の復習",
        ("ja", "mastered_1") => "最初の種",
        ("ja", "mastered_10") => "10の種",
        ("ja", "mastered_25") => "25の種",
        ("ja", "mastered_50") => "50の種",
        ("ja", "streak_3") => "3日連続",
        ("ja", "streak_7") => "1週間",
        ("ja", "streak_30") => "1か月",
        ("ja", "lesson_1") => "レッスン制覇",
        ("ja", "lesson_5") => "5レッスン制覇",
        ("ja", "level_5") => "レベル5",
        ("ja", "weekly_goal_1") => "目標達成",
        ("es", "first_step") => "Primer paso",
        ("es", "reviews_25") => "Calentando",
        ("es", "reviews_100") => "Cien repasos",
        ("es", "reviews_250") => "Constancia",
        ("es", "reviews_500") => "Medio millar",
        ("es", "mastered_1") => "Primera semilla",
        ("es", "mastered_10") => "Diez semillas",
        ("es", "mastered_25") => "Veinticinco semillas",
        ("es", "mastered_50") => "Cincuenta semillas",
        ("es", "streak_3") => "Tres días",
        ("es", "streak_7") => "Una semana",
        ("es", "streak_30") => "Un mes",
        ("es", "lesson_1") => "Lección dominada",
        ("es", "lesson_5") => "Cinco lecciones",
        ("es", "level_5") => "Nivel 5",
        ("es", "weekly_goal_1") => "Objetivo cumplido",
        _ => key,
    };
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Claves del catálogo de logros (`db::ACHIEVEMENT_CATALOG`).
    const ACHIEVEMENT_KEYS: [&str; 16] = [
        "first_step",
        "reviews_25",
        "reviews_100",
        "reviews_250",
        "reviews_500",
        "mastered_1",
        "mastered_10",
        "mastered_25",
        "mastered_50",
        "streak_3",
        "streak_7",
        "streak_30",
        "lesson_1",
        "lesson_5",
        "level_5",
        "weekly_goal_1",
    ];

    const LANGUAGES: [&str; 3] = ["en", "ja", "es"];

    #[test]
    fn normalizes_unknown_languages_to_spanish() {
        assert_eq!(lang("en"), "en");
        assert_eq!(lang("ja"), "ja");
        assert_eq!(lang("es"), "es");
        assert_eq!(lang("fr"), "es");
        assert_eq!(lang(""), "es");
        assert_eq!(lang("EN"), "es");
    }

    #[test]
    fn translates_static_labels() {
        assert_eq!(practice_now("en"), "Practice now");
        assert_eq!(practice_now("ja"), "今すぐ練習");
        assert_eq!(practice_now("es"), "Practicar ahora");

        assert_eq!(open_app("en"), "Open kotoba");
        assert_eq!(open_app("ja"), "kotoba を開く");
        assert_eq!(open_app("es"), "Abrir kotoba");

        assert_eq!(pause("en"), "Pause");
        assert_eq!(pause("ja"), "一時停止");
        assert_eq!(pause("es"), "Pausar");

        assert_eq!(resume("en"), "Resume");
        assert_eq!(resume("ja"), "再開");
        assert_eq!(resume("es"), "Reanudar");

        assert_eq!(quit("en"), "Quit");
        assert_eq!(quit("ja"), "終了");
        assert_eq!(quit("es"), "Salir");

        assert_eq!(mode("en"), "Mode");
        assert_eq!(mode("ja"), "表示モード");
        assert_eq!(mode("es"), "Modo");

        assert_eq!(toast_body("en"), "Tap to review a word");
        assert_eq!(toast_body("ja"), "タップして単語を復習");
        assert_eq!(toast_body("es"), "Toca para repasar una palabra");

        assert_eq!(weekly_title("en"), "Weekly goal completed");
        assert_eq!(weekly_title("ja"), "週間目標を達成しました");
        assert_eq!(weekly_title("es"), "Objetivo semanal completado");

        assert_eq!(achievement_title("en"), "Achievement unlocked");
        assert_eq!(achievement_title("ja"), "実績を解除しました");
        assert_eq!(achievement_title("es"), "Logro desbloqueado");
    }

    #[test]
    fn translates_tooltip() {
        assert_eq!(tooltip("en"), "kotoba — passive Japanese review");
        assert_eq!(tooltip("ja"), "kotoba — 受動的な日本語復習");
        assert_eq!(tooltip("es"), "kotoba — repaso pasivo de japonés");
    }

    #[test]
    fn static_labels_fall_back_to_spanish() {
        assert_eq!(practice_now("fr"), practice_now("es"));
        assert_eq!(open_app("fr"), open_app("es"));
        assert_eq!(pause("fr"), pause("es"));
        assert_eq!(resume("fr"), resume("es"));
        assert_eq!(quit("fr"), quit("es"));
        assert_eq!(mode("fr"), mode("es"));
        assert_eq!(toast_body("fr"), toast_body("es"));
        assert_eq!(weekly_title("fr"), weekly_title("es"));
        assert_eq!(achievement_title("fr"), achievement_title("es"));
        assert_eq!(tooltip("fr"), tooltip("es"));
    }

    #[test]
    fn builds_weekly_body_without_saver() {
        assert_eq!(
            weekly_body("en", 42, false),
            "42 reviews this week. Well done!"
        );
        assert_eq!(
            weekly_body("ja", 42, false),
            "今週は42回の復習。よくできました！"
        );
        assert_eq!(
            weekly_body("es", 42, false),
            "42 repasos esta semana. ¡Bien hecho!"
        );
    }

    #[test]
    fn builds_weekly_body_with_saver() {
        assert_eq!(
            weekly_body("en", 7, true),
            "7 reviews this week. Well done! You earned a streak saver."
        );
        assert_eq!(
            weekly_body("ja", 7, true),
            "今週は7回の復習。よくできました！ ストリーク保護を1つ獲得しました。"
        );
        assert_eq!(
            weekly_body("es", 7, true),
            "7 repasos esta semana. ¡Bien hecho! Has ganado un salvavidas de racha."
        );
    }

    #[test]
    fn weekly_body_handles_zero_and_unknown_language() {
        assert_eq!(weekly_body("fr", 0, false), "0 repasos esta semana. ¡Bien hecho!");
        assert_eq!(weekly_body("fr", 0, true), weekly_body("es", 0, true));
    }

    #[test]
    fn builds_single_achievement_body() {
        assert_eq!(
            achievement_body_one("en", "First step"),
            "You unlocked “First step”"
        );
        assert_eq!(
            achievement_body_one("ja", "最初の一歩"),
            "「最初の一歩」を解除しました"
        );
        assert_eq!(
            achievement_body_one("es", "Primer paso"),
            "Has desbloqueado «Primer paso»"
        );
        assert_eq!(
            achievement_body_one("fr", "Primer paso"),
            "Has desbloqueado «Primer paso»"
        );
    }

    #[test]
    fn builds_multiple_achievement_body() {
        assert_eq!(
            achievement_body_many("en", 3),
            "You unlocked 3 achievements"
        );
        assert_eq!(achievement_body_many("ja", 3), "実績を3個解除しました");
        assert_eq!(
            achievement_body_many("es", 3),
            "Has desbloqueado 3 logros"
        );
        assert_eq!(
            achievement_body_many("fr", 3),
            "Has desbloqueado 3 logros"
        );
    }

    #[test]
    fn translates_every_achievement_key() {
        for key in ACHIEVEMENT_KEYS {
            for language in LANGUAGES {
                let name = achievement_name(language, key);
                assert!(
                    !name.is_empty(),
                    "nombre vacío para {language}/{key}"
                );
                assert_ne!(
                    name, key,
                    "falta la traducción {language} para el logro {key}"
                );
            }
        }
    }

    #[test]
    fn translates_known_achievement_names() {
        assert_eq!(achievement_name("es", "first_step"), "Primer paso");
        assert_eq!(achievement_name("en", "first_step"), "First step");
        assert_eq!(achievement_name("ja", "first_step"), "最初の一歩");

        assert_eq!(achievement_name("es", "reviews_500"), "Medio millar");
        assert_eq!(achievement_name("en", "reviews_500"), "Half a thousand");
        assert_eq!(achievement_name("ja", "reviews_500"), "500回の復習");

        assert_eq!(achievement_name("es", "weekly_goal_1"), "Objetivo cumplido");
        assert_eq!(achievement_name("en", "weekly_goal_1"), "Goal completed");
        assert_eq!(achievement_name("ja", "weekly_goal_1"), "目標達成");
    }

    #[test]
    fn achievement_names_fall_back_to_key_and_spanish() {
        assert_eq!(achievement_name("es", "unknown_key"), "unknown_key");
        assert_eq!(achievement_name("en", "unknown_key"), "unknown_key");
        assert_eq!(achievement_name("ja", "unknown_key"), "unknown_key");

        // Un idioma no soportado usa las traducciones en español.
        for key in ACHIEVEMENT_KEYS {
            assert_eq!(
                achievement_name("fr", key),
                achievement_name("es", key),
                "el idioma desconocido debería caer en español para {key}"
            );
        }
    }

    #[test]
    fn achievement_names_are_unique_per_language() {
        for language in LANGUAGES {
            let mut names: Vec<String> = ACHIEVEMENT_KEYS
                .iter()
                .map(|key| achievement_name(language, key))
                .collect();
            let total = names.len();
            names.sort();
            names.dedup();
            assert_eq!(
                names.len(),
                total,
                "hay nombres de logro duplicados en {language}"
            );
        }
    }
}
