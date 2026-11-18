use crate::db;
use crate::AppState;
use tauri::menu::{
    CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu,
};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub struct TrayState {
    pub pause_item: MenuItem<tauri::Wry>,
    pub popup_mode: CheckMenuItem<tauri::Wry>,
    pub widget_mode: CheckMenuItem<tauri::Wry>,
    pub toast_mode: CheckMenuItem<tauri::Wry>,
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let mode = app
        .state::<AppState>()
        .db
        .lock()
        .ok()
        .and_then(|conn| db::load_settings(&conn).ok())
        .map(|settings| settings.mode)
        .unwrap_or_else(|| "popup".to_string());

    let practice = MenuItem::with_id(app, "practice", "Practicar ahora", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Abrir kotoba", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pausar", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

    let popup_mode = CheckMenuItem::with_id(
        app,
        "mode_popup",
        "Popup",
        true,
        mode == "popup",
        None::<&str>,
    )?;
    let widget_mode = CheckMenuItem::with_id(
        app,
        "mode_widget",
        "Widget",
        true,
        mode == "widget",
        None::<&str>,
    )?;
    let toast_mode = CheckMenuItem::with_id(
        app,
        "mode_toast",
        "Toast",
        true,
        mode == "toast",
        None::<&str>,
    )?;
    let mode_menu = Submenu::with_items(
        app,
        "Modo",
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
        .tooltip("kotoba — repaso pasivo de japonés")
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
        pause_item: pause,
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
