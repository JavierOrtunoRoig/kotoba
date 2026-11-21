mod db;
mod i18n;
mod import;
mod media;
mod models;
mod scheduler;
mod shell;
mod text;
mod tray;

use models::{AnswerResult, Bootstrap, Challenge, ImportSummary, Settings, Stats};
use chrono::TimeZone;
use rusqlite::OptionalExtension;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub db_path: PathBuf,
    pub media_dir: PathBuf,
    pub popup_visible: AtomicBool,
    pub widget_visible: AtomicBool,
}

fn to_err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/// Alterna la dirección respecto a la última respuesta.
fn alternate_direction(last: Option<&str>) -> &'static str {
    match last {
        Some("jp_to_es") => "es_to_jp",
        _ => "jp_to_es",
    }
}

/// Calidad SM-2 según acierto, latencia y respuesta vacía.
fn review_quality(correct: bool, answer: &str, latency_ms: i64) -> u8 {
    if correct {
        if latency_ms <= 5_000 {
            5
        } else {
            4
        }
    } else if answer.trim().is_empty() {
        1
    } else {
        2
    }
}

#[tauri::command]
fn bootstrap(state: State<AppState>) -> Result<Bootstrap, String> {
    let conn = state.db.lock().map_err(to_err)?;
    let item_count = db::count_items(&conn).map_err(to_err)?;
    let settings = db::load_settings(&conn).map_err(to_err)?;
    let stats = db::stats(
        &conn,
        settings.include_patterns,
        settings.weekly_goal,
        settings.lesson_mastery_percent,
        settings.new_per_day,
    )
    .map_err(to_err)?;
    Ok(Bootstrap {
        item_count,
        settings,
        stats,
    })
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    let conn = state.db.lock().map_err(to_err)?;
    db::load_settings(&conn).map_err(to_err)
}

#[tauri::command]
fn update_settings(
    app: tauri::AppHandle,
    state: State<AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let (previous_mode, previous_language, previous_scale, saved) = {
        let conn = state.db.lock().map_err(to_err)?;
        let previous = db::load_settings(&conn).map_err(to_err)?;
        let saved = db::save_settings(&conn, &settings).map_err(to_err)?;
        (
            previous.mode,
            previous.language,
            previous.font_scale,
            saved,
        )
    };

    if previous_mode != saved.mode {
        let handle = app.clone();
        let mode = saved.mode.clone();
        std::thread::spawn(move || shell::apply_mode(&handle, &mode));
    }
    if previous_language != saved.language {
        crate::tray::sync_language(&app, &saved.language);
    }
    if previous_scale != saved.font_scale {
        shell::apply_font_scale(&app);
    }
    let _ = app.emit("settings:changed", &saved);
    Ok(saved)
}

#[tauri::command]
fn get_stats(state: State<AppState>) -> Result<Stats, String> {
    let conn = state.db.lock().map_err(to_err)?;
    let settings = db::load_settings(&conn).map_err(to_err)?;
    db::stats(
        &conn,
        settings.include_patterns,
        settings.weekly_goal,
        settings.lesson_mastery_percent,
        settings.new_per_day,
    )
    .map_err(to_err)
}

#[tauri::command]
fn import_deck(
    app: tauri::AppHandle,
    state: State<AppState>,
    path: String,
    mapping: Option<models::ImportMapping>,
) -> Result<ImportSummary, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("No existe el archivo {}", path.display()));
    }
    log::info!("importando {}", path.display());

    let window = app.get_webview_window("main");
    let mut progress = |phase: &str, current: i64, total: i64| {
        if let Some(window) = &window {
            let _ = window.emit(
                "import:progress",
                models::ImportProgress {
                    phase: phase.to_string(),
                    current,
                    total,
                },
            );
        }
    };

    let mut conn = state.db.lock().map_err(to_err)?;
    let summary = import::import_file(
        &path,
        &mut conn,
        &state.media_dir,
        &mut progress,
        mapping.as_ref(),
    )
    .map_err(to_err)?;
    let _ = db::cleanup_media(&conn, &state.media_dir);
    log::info!(
        "importación completada: {} notas, {} ítems, {} audios",
        summary.notes,
        summary.items_inserted,
        summary.audio_extracted
    );
    Ok(summary)
}

#[tauri::command]
fn inspect_import(path: String) -> Result<models::ImportInspection, String> {
    import::inspect_file(&PathBuf::from(path)).map_err(to_err)
}

#[tauri::command]
fn next_challenge(
    state: State<AppState>,
    exclude_id: Option<i64>,
) -> Result<Option<Challenge>, String> {
    let conn = state.db.lock().map_err(to_err)?;
    let settings = db::load_settings(&conn).map_err(to_err)?;
    let Some(item) = db::next_item(
        &conn,
        exclude_id.unwrap_or(0),
        settings.include_patterns,
        settings.new_per_day,
        settings.new_mix_percent,
        true,
    )
    .map_err(to_err)?
    else {
        return Ok(None);
    };

    let direction = if settings.direction_mode == "alternate" {
        alternate_direction(db::last_direction(&conn).map_err(to_err)?.as_deref())
    } else if rand::random::<bool>() {
        "jp_to_es"
    } else {
        "es_to_jp"
    };

    let due_count = db::due_count(&conn, settings.include_patterns).map_err(to_err)?;
    let new_count = db::new_count(&conn, settings.include_patterns).map_err(to_err)?;
    Ok(Some(Challenge {
        item,
        direction: direction.to_string(),
        due_count,
        new_count,
    }))
}

#[tauri::command]
fn submit_answer(
    app: tauri::AppHandle,
    state: State<AppState>,
    item_id: i64,
    direction: String,
    answer: String,
    latency_ms: i64,
) -> Result<AnswerResult, String> {
    let mut conn = state.db.lock().map_err(to_err)?;
    let item = db::item_by_id(&conn, item_id).map_err(to_err)?;
    let settings = db::load_settings(&conn).map_err(to_err)?;
    let week_before = db::reviews_this_week(&conn).map_err(to_err)?;

    let correct = if direction == "jp_to_es" {
        text::check_jp_to_es(&answer, &item)
    } else {
        text::check_es_to_jp(&answer, &item)
    };

    let quality = review_quality(correct, &answer, latency_ms);

    let outcome = db::record_review(&mut conn, item_id, quality, &direction, latency_ms)
        .map_err(to_err)?;
    if let Some(day) = db::maintain_streak(&conn, settings.streak_repair).map_err(to_err)? {
        log::info!("racha reparada con salvavidas: {day}");
    }
    let week_after = db::reviews_this_week(&conn).map_err(to_err)?;
    let mut saver_earned = false;
    if settings.weekly_goal > 0
        && week_before < settings.weekly_goal
        && week_after >= settings.weekly_goal
    {
        db::bump_weekly_goals_done(&conn).map_err(to_err)?;
        saver_earned = db::add_streak_saver(&conn).map_err(to_err)?;
    }
    let unlocked = db::sync_achievements(&conn, settings.lesson_mastery_percent).map_err(to_err)?;
    drop(conn);

    let allow_notifications = settings.milestone_notifications
        && !settings.paused
        && !scheduler::in_dnd(&settings)
        && !(settings.suppress_fullscreen && shell::fullscreen_or_busy());

    if allow_notifications
        && settings.weekly_goal > 0
        && week_before < settings.weekly_goal
        && week_after >= settings.weekly_goal
    {
        let body = i18n::weekly_body(&settings.language, week_after, saver_earned);
        shell::show_notification(
            &app,
            i18n::weekly_title(&settings.language),
            &body,
        );
        log::info!("objetivo semanal completado: {week_after} repasos");
    }

    if allow_notifications && !unlocked.is_empty() {
        let body = if unlocked.len() == 1 {
            let title = i18n::achievement_name(&settings.language, &unlocked[0].key);
            i18n::achievement_body_one(&settings.language, &title)
        } else {
            i18n::achievement_body_many(&settings.language, unlocked.len())
        };
        shell::show_notification(
            &app,
            i18n::achievement_title(&settings.language),
            &body,
        );
        let keys: Vec<&str> = unlocked.iter().map(|a| a.key.as_str()).collect();
        log::info!("logros desbloqueados: {}", keys.join(", "));
    }

    let expected = if direction == "jp_to_es" {
        text::meaning_variants(&item)
    } else {
        text::japanese_variants(&item)
    };

    Ok(AnswerResult {
        correct,
        expected,
        reading: item.reading,
        meaning: item.meaning,
        kanji_gloss: item.kanji_gloss,
        has_audio: item.has_audio,
        next_interval_minutes: outcome.next_interval_minutes,
        xp_gained: outcome.xp_gained,
        xp_total: outcome.xp_total,
        level: outcome.level,
        leveled_up: outcome.leveled_up,
        unlocked,
        saver_earned,
    })
}

#[tauri::command]
fn get_audio(state: State<AppState>, item_id: i64) -> Result<Option<Vec<u8>>, String> {
    let conn = state.db.lock().map_err(to_err)?;
    let audio_file: Option<String> = conn
        .query_row(
            "SELECT audio_file FROM items WHERE id = ?1",
            [item_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(to_err)?;
    drop(conn);

    if let Some(file) = audio_file.filter(|value| !value.is_empty()) {
        let path = state.media_dir.join(file);
        if path.exists() {
            return std::fs::read(path).map(Some).map_err(to_err);
        }
    }
    Ok(None)
}

#[tauri::command]
fn reset_progress(state: State<AppState>) -> Result<Stats, String> {
    let conn = state.db.lock().map_err(to_err)?;
    db::reset_progress(&conn).map_err(to_err)?;
    let settings = db::load_settings(&conn).map_err(to_err)?;
    db::stats(
        &conn,
        settings.include_patterns,
        settings.weekly_goal,
        settings.lesson_mastery_percent,
        settings.new_per_day,
    )
    .map_err(to_err)
}

#[tauri::command]
fn clear_library(state: State<AppState>) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(to_err)?;
    db::clear_library(&mut conn).map_err(to_err)?;
    let _ = db::cleanup_media(&conn, &state.media_dir);
    Ok(())
}

#[tauri::command]
fn export_backup(state: State<AppState>, destination: String) -> Result<String, String> {
    let destination = PathBuf::from(destination);
    if !destination.is_dir() {
        return Err("La carpeta de destino no existe".to_string());
    }

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let folder = destination.join(format!("kotoba-backup-{timestamp}"));
    std::fs::create_dir_all(&folder).map_err(to_err)?;
    log::info!("exportando copia a {}", folder.display());

    let conn = state.db.lock().map_err(to_err)?;
    let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
    std::fs::copy(&state.db_path, folder.join("kotoba.sqlite3")).map_err(to_err)?;
    copy_media(&state.media_dir, &folder.join("media")).map_err(to_err)?;

    Ok(folder.to_string_lossy().to_string())
}

fn copy_media(source: &std::path::Path, target: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    if !source.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            std::fs::copy(entry.path(), target.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn restore_backup(
    app: tauri::AppHandle,
    state: State<AppState>,
    source: String,
) -> Result<Settings, String> {
    let source = PathBuf::from(source);
    let backup_db = source.join("kotoba.sqlite3");
    if !backup_db.is_file() {
        return Err("La carpeta no contiene kotoba.sqlite3".to_string());
    }

    let source_db = std::fs::canonicalize(&backup_db).map_err(to_err)?;
    let live_db = std::fs::canonicalize(&state.db_path).map_err(to_err)?;
    if source_db == live_db {
        return Err("No se puede restaurar la base de datos activa".to_string());
    }
    log::info!("restaurando copia desde {}", source.display());

    {
        let test = rusqlite::Connection::open_with_flags(
            &backup_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(to_err)?;
        test.query_row("SELECT COUNT(*) FROM items", [], |_| Ok(()))
            .map_err(|_| "La base de la copia no es válida".to_string())?;
    }

    let backup_media = source.join("media");
    let mut conn = state.db.lock().map_err(to_err)?;

    // Cerrar la base viva antes de reemplazar el archivo.
    let placeholder = rusqlite::Connection::open_in_memory().map_err(to_err)?;
    let old = std::mem::replace(&mut *conn, placeholder);
    drop(old);

    std::fs::copy(&backup_db, &state.db_path).map_err(to_err)?;
    let _ = std::fs::remove_file(state.db_path.with_extension("sqlite3-wal"));
    let _ = std::fs::remove_file(state.db_path.with_extension("sqlite3-shm"));

    if backup_media.is_dir() {
        let _ = std::fs::remove_dir_all(&state.media_dir);
        std::fs::create_dir_all(&state.media_dir).map_err(to_err)?;
        copy_media(&backup_media, &state.media_dir).map_err(to_err)?;
    }

    let new_conn = db::open(&state.db_path).map_err(to_err)?;
    let settings = db::load_settings(&new_conn).map_err(to_err)?;
    *conn = new_conn;
    drop(conn);

    shell::close_popup(&app);
    shell::hide_widget(&app);
    crate::tray::sync_mode(&app, &settings.mode);
    crate::tray::sync_pause(&app, settings.paused);
    shell::apply_mode(&app, &settings.mode);
    let _ = app.emit("settings:changed", &settings);
    let _ = app.emit("library:changed", ());
    log::info!("copia restaurada desde {}", source.display());
    Ok(settings)
}

#[tauri::command]
fn export_history(state: State<AppState>, destination: String) -> Result<String, String> {
    let destination = PathBuf::from(destination);
    if let Some(parent) = destination.parent() {
        if !parent.as_os_str().is_empty() && !parent.is_dir() {
            return Err("La carpeta de destino no existe".to_string());
        }
    }
    log::info!("exportando historial a {}", destination.display());

    let conn = state.db.lock().map_err(to_err)?;
    let mut stmt = conn
        .prepare(
            "SELECT r.answered_at, r.item_id, i.reading, i.meaning, r.direction,
                    r.result, r.quality, r.latency_ms
             FROM review_log r
             JOIN items i ON i.id = r.item_id
             ORDER BY r.answered_at",
        )
        .map_err(to_err)?;
    let mut rows = stmt.query([]).map_err(to_err)?;

    let mut csv = String::from(
        "answered_at,item_id,reading,meaning,direction,result,quality,latency_ms\n",
    );
    while let Some(row) = rows.next().map_err(to_err)? {
        let answered_at: i64 = row.get(0).map_err(to_err)?;
        let item_id: i64 = row.get(1).map_err(to_err)?;
        let reading: String = row.get(2).map_err(to_err)?;
        let meaning: String = row.get(3).map_err(to_err)?;
        let direction: String = row.get(4).map_err(to_err)?;
        let result: i64 = row.get(5).map_err(to_err)?;
        let quality: i64 = row.get(6).map_err(to_err)?;
        let latency_ms: i64 = row.get(7).map_err(to_err)?;

        let answered = chrono::Local
            .timestamp_opt(answered_at, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| answered_at.to_string());

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            csv_escape(&answered),
            item_id,
            csv_escape(&reading),
            csv_escape(&meaning),
            csv_escape(&direction),
            result,
            quality,
            latency_ms
        ));
    }

    std::fs::write(&destination, csv).map_err(to_err)?;
    Ok(destination.to_string_lossy().to_string())
}

fn csv_escape(value: &str) -> String {
    if value.chars().any(|c| matches!(c, ',' | '"' | '\n' | '\r')) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[tauri::command]
fn practice_now(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mode = {
        let conn = state.db.lock().map_err(to_err)?;
        db::load_settings(&conn).map_err(to_err)?.mode
    };

    // Crear ventanas desde el hilo del comando puede bloquear el webview.
    std::thread::spawn(move || match mode.as_str() {
        "widget" => {
            let _ = shell::show_widget(&app);
        }
        "toast" => shell::show_toast(&app),
        _ => {
            let _ = shell::show_popup(&app);
        }
    });
    Ok(())
}

#[tauri::command]
fn close_popup(app: tauri::AppHandle) {
    shell::close_popup(&app);
}

#[tauri::command]
fn hide_widget(app: tauri::AppHandle) {
    shell::hide_widget(&app);
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    shell::show_main(&app);
}

#[tauri::command]
fn open_logs(app: tauri::AppHandle) -> Result<(), String> {
    let dir = app.path().app_log_dir().map_err(to_err)?;
    std::fs::create_dir_all(&dir).map_err(to_err)?;
    std::process::Command::new("explorer")
        .arg(&dir)
        .spawn()
        .map_err(to_err)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("kotoba".into()),
                    },
                ))
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let media_dir = data_dir.join("media");
            std::fs::create_dir_all(&media_dir)?;

            let db_path = data_dir.join("kotoba.sqlite3");
            let conn = db::open(&db_path)?;
            let has_items = db::count_items(&conn)? > 0;
            let settings = db::load_settings(&conn)?;

            app.manage(AppState {
                db: Mutex::new(conn),
                db_path,
                media_dir,
                popup_visible: AtomicBool::new(false),
                widget_visible: AtomicBool::new(false),
            });

            {
                use tauri_plugin_notification::NotificationExt;
                let handle = app.handle().clone();
                app.notification().on_action(move |action| {
                    if action.action_id() == "tap" {
                        let _ = shell::show_popup(&handle);
                    }
                })?;
            }

            tray::setup(app.handle())?;
            scheduler::start(app.handle().clone());

            if !has_items {
                shell::show_main(app.handle());
            } else if settings.mode == "widget" {
                let _ = shell::show_widget(app.handle());
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if matches!(window.label(), "main" | "popup" | "widget") {
                    api.prevent_close();
                    let _ = window.hide();
                    if let Some(state) = window.app_handle().try_state::<AppState>() {
                        match window.label() {
                            "popup" => state.popup_visible.store(false, Ordering::Relaxed),
                            "widget" => state.widget_visible.store(false, Ordering::Relaxed),
                            _ => {}
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            get_settings,
            update_settings,
            get_stats,
            import_deck,
            inspect_import,
            next_challenge,
            submit_answer,
            get_audio,
            reset_progress,
            clear_library,
            export_backup,
            restore_backup,
            export_history,
            practice_now,
            close_popup,
            hide_widget,
            show_main_window,
            open_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn alternates_against_the_last_direction() {
        assert_eq!(alternate_direction(Some("jp_to_es")), "es_to_jp");
        assert_eq!(alternate_direction(Some("es_to_jp")), "jp_to_es");
    }

    #[test]
    fn alternate_defaults_to_japanese_to_spanish() {
        assert_eq!(alternate_direction(None), "jp_to_es");
        assert_eq!(alternate_direction(Some("")), "jp_to_es");
        assert_eq!(alternate_direction(Some("unknown")), "jp_to_es");
    }

    #[test]
    fn fast_correct_answers_get_top_quality() {
        assert_eq!(review_quality(true, "四", 0), 5);
        assert_eq!(review_quality(true, "四", 5_000), 5);
    }

    #[test]
    fn slow_correct_answers_lose_one_point() {
        assert_eq!(review_quality(true, "四", 5_001), 4);
        assert_eq!(review_quality(true, "四", 60_000), 4);
    }

    #[test]
    fn empty_wrong_answers_get_lowest_quality() {
        assert_eq!(review_quality(false, "", 1_000), 1);
        assert_eq!(review_quality(false, "   ", 1_000), 1);
        assert_eq!(review_quality(false, "\n\t", 1_000), 1);
    }

    #[test]
    fn wrong_answers_get_quality_two() {
        assert_eq!(review_quality(false, "ご", 1_000), 2);
        assert_eq!(review_quality(false, "ご", 60_000), 2);
    }

    #[test]
    fn correct_answer_wins_over_empty_check() {
        assert_eq!(review_quality(true, "", 1_000), 5);
    }

    #[test]
    fn csv_escape_keeps_simple_values() {
        assert_eq!(csv_escape("escuela"), "escuela");
        assert_eq!(csv_escape(""), "");
        assert_eq!(csv_escape("日本語"), "日本語");
    }

    #[test]
    fn csv_escape_quotes_separators_and_newlines() {
        assert_eq!(csv_escape("a,b"), "\"a,b\"");
        assert_eq!(csv_escape("a\nb"), "\"a\nb\"");
        assert_eq!(csv_escape("a\rb"), "\"a\rb\"");
    }

    #[test]
    fn csv_escape_doubles_inner_quotes() {
        assert_eq!(csv_escape("decir \"hola\""), "\"decir \"\"hola\"\"\"");
    }

    #[test]
    fn copy_media_copies_top_level_files() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let target = dir.path().join("target");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("a.mp3"), b"audio").unwrap();
        fs::write(source.join("b.png"), b"image").unwrap();

        copy_media(&source, &target).unwrap();

        assert_eq!(fs::read(target.join("a.mp3")).unwrap(), b"audio");
        assert_eq!(fs::read(target.join("b.png")).unwrap(), b"image");
    }

    #[test]
    fn copy_media_skips_subdirectories() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let target = dir.path().join("target");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested").join("c.mp3"), b"audio").unwrap();
        fs::write(source.join("top.mp3"), b"audio").unwrap();

        copy_media(&source, &target).unwrap();

        assert!(target.join("top.mp3").is_file());
        assert!(!target.join("nested").exists());
    }

    #[test]
    fn copy_media_handles_missing_source() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");

        copy_media(&dir.path().join("missing"), &target).unwrap();

        assert!(target.is_dir());
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    }

    #[test]
    fn copy_media_creates_nested_target() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let target = dir.path().join("a").join("b");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("x.mp3"), b"audio").unwrap();

        copy_media(&source, &target).unwrap();

        assert_eq!(fs::read(target.join("x.mp3")).unwrap(), b"audio");
    }
}
