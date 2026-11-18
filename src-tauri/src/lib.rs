mod db;
mod import;
mod media;
mod models;
mod scheduler;
mod shell;
mod text;
mod tray;

use models::{AnswerResult, Bootstrap, Challenge, ImportSummary, Settings, Stats};
use rusqlite::OptionalExtension;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub media_dir: PathBuf,
    pub popup_visible: AtomicBool,
    pub widget_visible: AtomicBool,
}

fn to_err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[tauri::command]
fn bootstrap(state: State<AppState>) -> Result<Bootstrap, String> {
    let conn = state.db.lock().map_err(to_err)?;
    let item_count = db::count_items(&conn).map_err(to_err)?;
    let settings = db::load_settings(&conn).map_err(to_err)?;
    let stats = db::stats(&conn, settings.include_patterns).map_err(to_err)?;
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
    let (previous_mode, saved) = {
        let conn = state.db.lock().map_err(to_err)?;
        let previous = db::load_settings(&conn).map_err(to_err)?;
        let saved = db::save_settings(&conn, &settings).map_err(to_err)?;
        (previous.mode, saved)
    };

    if previous_mode != saved.mode {
        let handle = app.clone();
        let mode = saved.mode.clone();
        std::thread::spawn(move || shell::apply_mode(&handle, &mode));
    }
    let _ = app.emit("settings:changed", &saved);
    Ok(saved)
}

#[tauri::command]
fn get_stats(state: State<AppState>) -> Result<Stats, String> {
    let conn = state.db.lock().map_err(to_err)?;
    let settings = db::load_settings(&conn).map_err(to_err)?;
    db::stats(&conn, settings.include_patterns).map_err(to_err)
}

#[tauri::command]
fn import_deck(state: State<AppState>, path: String) -> Result<ImportSummary, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("No existe el archivo {}", path.display()));
    }
    let mut conn = state.db.lock().map_err(to_err)?;
    import::import_file(&path, &mut conn, &state.media_dir).map_err(to_err)
}

#[tauri::command]
fn next_challenge(
    state: State<AppState>,
    exclude_id: Option<i64>,
) -> Result<Option<Challenge>, String> {
    let conn = state.db.lock().map_err(to_err)?;
    let settings = db::load_settings(&conn).map_err(to_err)?;
    let Some(item) = db::next_item(&conn, exclude_id.unwrap_or(0), settings.include_patterns)
        .map_err(to_err)?
    else {
        return Ok(None);
    };

    let direction = if settings.direction_mode == "alternate" {
        match db::last_direction(&conn).map_err(to_err)?.as_deref() {
            Some("jp_to_es") => "es_to_jp",
            _ => "jp_to_es",
        }
    } else if rand::random::<bool>() {
        "jp_to_es"
    } else {
        "es_to_jp"
    };

    let due_count = db::due_count(&conn, settings.include_patterns).map_err(to_err)?;
    Ok(Some(Challenge {
        item,
        direction: direction.to_string(),
        due_count,
    }))
}

#[tauri::command]
fn submit_answer(
    state: State<AppState>,
    item_id: i64,
    direction: String,
    answer: String,
    latency_ms: i64,
) -> Result<AnswerResult, String> {
    let mut conn = state.db.lock().map_err(to_err)?;
    let item = db::item_by_id(&conn, item_id).map_err(to_err)?;

    let correct = if direction == "jp_to_es" {
        text::check_jp_to_es(&answer, &item)
    } else {
        text::check_es_to_jp(&answer, &item)
    };

    let quality = if correct {
        if latency_ms <= 5_000 {
            5
        } else {
            4
        }
    } else if answer.trim().is_empty() {
        1
    } else {
        2
    };

    let next_interval_minutes =
        db::record_review(&mut conn, item_id, quality, &direction, latency_ms).map_err(to_err)?;

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
        next_interval_minutes,
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
    db::stats(&conn, settings.include_patterns).map_err(to_err)
}

#[tauri::command]
fn clear_library(state: State<AppState>) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(to_err)?;
    db::clear_library(&mut conn).map_err(to_err)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
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
            next_challenge,
            submit_answer,
            get_audio,
            reset_progress,
            clear_library,
            practice_now,
            close_popup,
            hide_widget,
            show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
