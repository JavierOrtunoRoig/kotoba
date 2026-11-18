use crate::db;
use crate::tray::TrayState;
use crate::AppState;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn show_popup(app: &AppHandle) -> tauri::Result<()> {
    let window = match app.get_webview_window("popup") {
        Some(window) => window,
        None => WebviewWindowBuilder::new(app, "popup", WebviewUrl::App("index.html".into()))
            .title("kotoba")
            .inner_size(392.0, 280.0)
            .min_inner_size(360.0, 240.0)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .focused(false)
            .visible(false)
            .build()?,
    };

    position_popup(&window)?;
    window.show()?;
    let _ = window.emit("challenge:show", ());
    Ok(())
}

pub fn close_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.hide();
    }
}

fn position_popup(window: &WebviewWindow) -> tauri::Result<()> {
    let monitor = window.current_monitor()?.or(window.primary_monitor()?);
    if let Some(monitor) = monitor {
        let size = window.outer_size()?;
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();
        let margin = 24i32;
        let taskbar = 52i32;
        let x = monitor_position.x + monitor_size.width as i32 - size.width as i32 - margin;
        let y = monitor_position.y + monitor_size.height as i32 - size.height as i32 - margin - taskbar;
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

pub fn toggle_pause(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let Ok(conn) = state.db.lock() else {
        return false;
    };
    let mut settings = db::load_settings(&conn).unwrap_or_default();
    settings.paused = !settings.paused;
    let _ = db::save_settings(&conn, &settings);
    drop(conn);

    if let Some(tray) = app.try_state::<TrayState>() {
        let _ = tray
            .pause_item
            .set_text(if settings.paused { "Reanudar" } else { "Pausar" });
    }
    let _ = app.emit("settings:changed", &settings);
    settings.paused
}
