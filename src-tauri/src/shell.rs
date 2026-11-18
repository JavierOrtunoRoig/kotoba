use crate::db;
use crate::tray::TrayState;
use crate::AppState;
use std::sync::atomic::Ordering;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

const POPUP_WIDTH: f64 = 392.0;
const POPUP_HEIGHT: f64 = 280.0;
const WIDGET_WIDTH: f64 = 340.0;
const WIDGET_HEIGHT: f64 = 248.0;
const TASKBAR_OFFSET: i32 = 52;

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn show_popup(app: &AppHandle) -> tauri::Result<()> {
    let window = ensure_window(app, "popup", POPUP_WIDTH, POPUP_HEIGHT)?;
    position_bottom_right(&window)?;

    if let Some(state) = app.try_state::<AppState>() {
        state.popup_visible.store(true, Ordering::Relaxed);
    }

    show_without_activation(&window)?;
    let _ = window.emit("challenge:show", ());
    Ok(())
}

pub fn show_widget(app: &AppHandle) -> tauri::Result<()> {
    let window = ensure_window(app, "widget", WIDGET_WIDTH, WIDGET_HEIGHT)?;
    position_bottom_right(&window)?;

    if let Some(state) = app.try_state::<AppState>() {
        state.widget_visible.store(true, Ordering::Relaxed);
    }

    show_without_activation(&window)?;
    let _ = window.emit("challenge:show", ());
    Ok(())
}

pub fn close_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = window.hide();
    }
    if let Some(state) = app.try_state::<AppState>() {
        state.popup_visible.store(false, Ordering::Relaxed);
    }
}

pub fn hide_widget(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("widget") {
        let _ = window.hide();
    }
    if let Some(state) = app.try_state::<AppState>() {
        state.widget_visible.store(false, Ordering::Relaxed);
    }
}

/// Notificación nativa. El click se gestiona en `setup` mediante `on_action`.
pub fn show_toast(app: &AppHandle) {
    use tauri_plugin_notification::NotificationExt;

    let _ = app
        .notification()
        .builder()
        .title("kotoba")
        .body("Toca para repasar una palabra")
        .show();
}

/// Aplica un cambio de modo a las ventanas y a la bandeja.
pub fn apply_mode(app: &AppHandle, mode: &str) {
    match mode {
        "widget" => {
            close_popup(app);
            if let Err(error) = show_widget(app) {
                log::warn!("no se pudo mostrar el widget: {error}");
            }
        }
        "toast" => {
            close_popup(app);
            hide_widget(app);
        }
        _ => {
            hide_widget(app);
        }
    }
    crate::tray::sync_mode(app, mode);
}

/// Cambia el modo desde la bandeja y persiste el ajuste.
pub fn set_mode(app: &AppHandle, mode: &str) -> bool {
    let state = app.state::<AppState>();
    let Ok(conn) = state.db.lock() else {
        return false;
    };
    let mut settings = db::load_settings(&conn).unwrap_or_default();
    if settings.mode == mode {
        return true;
    }
    settings.mode = mode.to_string();
    let settings = db::save_settings(&conn, &settings).unwrap_or(settings);
    drop(conn);

    apply_mode(app, &settings.mode);
    let _ = app.emit("settings:changed", &settings);
    true
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

/// Detecta pantalla completa, presentaciones, juegos o modo silencio de Windows.
#[cfg(target_os = "windows")]
pub fn fullscreen_or_busy() -> bool {
    use windows_sys::Win32::UI::Shell::{
        SHQueryUserNotificationState, QUNS_ACCEPTS_NOTIFICATIONS,
    };

    let mut state = QUNS_ACCEPTS_NOTIFICATIONS;
    let result = unsafe { SHQueryUserNotificationState(&mut state) };
    if result < 0 {
        return false;
    }
    state != QUNS_ACCEPTS_NOTIFICATIONS
}

#[cfg(not(target_os = "windows"))]
pub fn fullscreen_or_busy() -> bool {
    false
}

fn ensure_window(
    app: &AppHandle,
    label: &str,
    width: f64,
    height: f64,
) -> tauri::Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(label) {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("kotoba")
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()
}

fn position_bottom_right(window: &WebviewWindow) -> tauri::Result<()> {
    let monitor = window.current_monitor()?.or(window.primary_monitor()?);
    if let Some(monitor) = monitor {
        let size = window.outer_size()?;
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();
        let margin = 24i32;
        let x = monitor_position.x + monitor_size.width as i32 - size.width as i32 - margin;
        let y = monitor_position.y + monitor_size.height as i32 - size.height as i32 - margin
            - TASKBAR_OFFSET;
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

/// Muestra la ventana y devuelve el foco a la ventana anterior.
///
/// Tauri necesita `show()` para inicializar/navegar el webview; después se
/// restaura el foco y se fuerza el topmost sin activar.
#[cfg(target_os = "windows")]
fn show_without_activation(window: &WebviewWindow) -> tauri::Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, SetForegroundWindow, SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    };

    let previous = unsafe { GetForegroundWindow() };
    window.show()?;

    let hwnd = window.hwnd()?;
    unsafe {
        SetWindowPos(
            hwnd.0,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
        if !previous.is_null() {
            SetForegroundWindow(previous);
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn show_without_activation(window: &WebviewWindow) -> tauri::Result<()> {
    window.show()
}
