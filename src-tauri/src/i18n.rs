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
