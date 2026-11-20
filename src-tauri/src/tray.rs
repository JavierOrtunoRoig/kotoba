use crate::{db, i18n};
use crate::AppState;
use tauri::menu::{
    CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu,
};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub struct TrayState {
    pub practice_item: MenuItem<tauri::Wry>,
    pub open_item: MenuItem<tauri::Wry>,
    pub pause_item: MenuItem<tauri::Wry>,
    pub quit_item: MenuItem<tauri::Wry>,
    pub mode_menu: Submenu<tauri::Wry>,
    pub popup_mode: CheckMenuItem<tauri::Wry>,
    pub widget_mode: CheckMenuItem<tauri::Wry>,
    pub toast_mode: CheckMenuItem<tauri::Wry>,
}

fn current_settings(app: &AppHandle) -> crate::models::Settings {
    app.state::<AppState>()
        .db
        .lock()
        .ok()
        .and_then(|conn| db::load_settings(&conn).ok())
        .unwrap_or_default()
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let settings = current_settings(app);
    let language = settings.language.as_str();

    let practice = MenuItem::with_id(
        app,
        "practice",
        i18n::practice_now(language),
        true,
        None::<&str>,
    )?;
    let open = MenuItem::with_id(app, "open", i18n::open_app(language), true, None::<&str>)?;
    let pause = MenuItem::with_id(
        app,
        "pause",
        if settings.paused {
            i18n::resume(language)
        } else {
            i18n::pause(language)
        },
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", i18n::quit(language), true, None::<&str>)?;

    let popup_mode = CheckMenuItem::with_id(
        app,
        "mode_popup",
        "Popup",
        true,
        settings.mode == "popup",
        None::<&str>,
    )?;
    let widget_mode = CheckMenuItem::with_id(
        app,
        "mode_widget",
        "Widget",
        true,
        settings.mode == "widget",
        None::<&str>,
    )?;
    let toast_mode = CheckMenuItem::with_id(
        app,
        "mode_toast",
        "Toast",
        true,
        settings.mode == "toast",
        None::<&str>,
    )?;
    let mode_menu = Submenu::with_items(
        app,
        i18n::mode(language),
        true,
        &[&popup_mode, &widget_mode, &toast_mode],
    )?;

    let separator_one = PredefinedMenuItem::separator(app)?;
    let separator_two = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &practice,
            &open,
            &mode_menu,
            &separator_one,
            &pause,
            &separator_two,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip(i18n::tooltip(language))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "practice" => {
                let _ = crate::shell::show_popup(app);
            }
            "open" => crate::shell::show_main(app),
            "pause" => {
                crate::shell::toggle_pause(app);
            }
            "mode_popup" => {
                crate::shell::set_mode(app, "popup");
            }
            "mode_widget" => {
                crate::shell::set_mode(app, "widget");
            }
            "mode_toast" => {
                crate::shell::set_mode(app, "toast");
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::shell::show_main(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    app.manage(TrayState {
        practice_item: practice,
        open_item: open,
        pause_item: pause,
        quit_item: quit,
        mode_menu,
        popup_mode,
        widget_mode,
        toast_mode,
    });
    Ok(())
}

pub fn sync_mode(app: &AppHandle, mode: &str) {
    if let Some(tray) = app.try_state::<TrayState>() {
        let _ = tray.popup_mode.set_checked(mode == "popup");
        let _ = tray.widget_mode.set_checked(mode == "widget");
        let _ = tray.toast_mode.set_checked(mode == "toast");
    }
}

pub fn sync_pause(app: &AppHandle, paused: bool) {
    let language = current_settings(app).language;
    if let Some(tray) = app.try_state::<TrayState>() {
        let _ = tray.pause_item.set_text(if paused {
            i18n::resume(&language)
        } else {
            i18n::pause(&language)
        });
    }
}

/// Actualiza los textos de la bandeja al cambiar el idioma.
pub fn sync_language(app: &AppHandle, language: &str) {
    let paused = current_settings(app).paused;
    if let Some(tray) = app.try_state::<TrayState>() {
        let _ = tray.practice_item.set_text(i18n::practice_now(language));
        let _ = tray.open_item.set_text(i18n::open_app(language));
        let _ = tray.pause_item.set_text(if paused {
            i18n::resume(language)
        } else {
            i18n::pause(language)
        });
        let _ = tray.quit_item.set_text(i18n::quit(language));
        let _ = tray.mode_menu.set_text(i18n::mode(language));
    }
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(i18n::tooltip(language)));
    }
}
