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

/// Label for the pause item based on the current state.
fn pause_label(language: &str, paused: bool) -> &'static str {
    if paused {
        i18n::resume(language)
    } else {
        i18n::pause(language)
    }
}

/// Check mark for each submenu mode: (popup, widget, toast).
fn mode_flags(mode: &str) -> (bool, bool, bool) {
    (mode == "popup", mode == "widget", mode == "toast")
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
        pause_label(language, settings.paused),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", i18n::quit(language), true, None::<&str>)?;

    let (popup_checked, widget_checked, toast_checked) = mode_flags(&settings.mode);
    let popup_mode = CheckMenuItem::with_id(
        app,
        "mode_popup",
        "Popup",
        true,
        popup_checked,
        None::<&str>,
    )?;
    let widget_mode = CheckMenuItem::with_id(
        app,
        "mode_widget",
        "Widget",
        true,
        widget_checked,
        None::<&str>,
    )?;
    let toast_mode = CheckMenuItem::with_id(
        app,
        "mode_toast",
        "Toast",
        true,
        toast_checked,
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
        let (popup, widget, toast) = mode_flags(mode);
        let _ = tray.popup_mode.set_checked(popup);
        let _ = tray.widget_mode.set_checked(widget);
        let _ = tray.toast_mode.set_checked(toast);
    }
}

pub fn sync_pause(app: &AppHandle, paused: bool) {
    let language = current_settings(app).language;
    if let Some(tray) = app.try_state::<TrayState>() {
        let _ = tray.pause_item.set_text(pause_label(&language, paused));
    }
}

/// Updates the tray texts when the language changes.
pub fn sync_language(app: &AppHandle, language: &str) {
    let paused = current_settings(app).paused;
    if let Some(tray) = app.try_state::<TrayState>() {
        let _ = tray.practice_item.set_text(i18n::practice_now(language));
        let _ = tray.open_item.set_text(i18n::open_app(language));
        let _ = tray.pause_item.set_text(pause_label(language, paused));
        let _ = tray.quit_item.set_text(i18n::quit(language));
        let _ = tray.mode_menu.set_text(i18n::mode(language));
    }
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(i18n::tooltip(language)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODES: [&str; 3] = ["popup", "widget", "toast"];

    #[test]
    fn pause_label_shows_resume_when_paused() {
        assert_eq!(pause_label("es", true), "Reanudar");
        assert_eq!(pause_label("en", true), "Resume");
        assert_eq!(pause_label("ja", true), "再開");
    }

    #[test]
    fn pause_label_shows_pause_when_running() {
        assert_eq!(pause_label("es", false), "Pausar");
        assert_eq!(pause_label("en", false), "Pause");
        assert_eq!(pause_label("ja", false), "一時停止");
    }

    #[test]
    fn pause_label_falls_back_to_spanish() {
        assert_eq!(pause_label("fr", false), pause_label("es", false));
        assert_eq!(pause_label("fr", true), pause_label("es", true));
    }

    #[test]
    fn mode_flags_checks_only_the_active_mode() {
        assert_eq!(mode_flags("popup"), (true, false, false));
        assert_eq!(mode_flags("widget"), (false, true, false));
        assert_eq!(mode_flags("toast"), (false, false, true));
    }

    #[test]
    fn mode_flags_have_exactly_one_check_for_known_modes() {
        for mode in MODES {
            let checked = mode_flags(mode);
            let count = [checked.0, checked.1, checked.2]
                .iter()
                .filter(|value| **value)
                .count();
            assert_eq!(count, 1, "modo {mode} debería marcar un solo ítem");
        }
    }

    #[test]
    fn mode_flags_ignore_unknown_modes() {
        for mode in ["", "Popup", "WIDGET", "unknown"] {
            assert_eq!(mode_flags(mode), (false, false, false), "modo {mode}");
        }
    }
}
