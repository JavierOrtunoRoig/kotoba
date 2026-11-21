use crate::db;
use crate::i18n;
use crate::AppState;
use std::sync::atomic::Ordering;
use tauri::{
    AppHandle, Emitter, Manager, LogicalSize, PhysicalPosition, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

const POPUP_WIDTH: f64 = 392.0;
const POPUP_HEIGHT: f64 = 280.0;
const WIDGET_WIDTH: f64 = 340.0;
const WIDGET_HEIGHT: f64 = 248.0;
const TASKBAR_OFFSET: i32 = 52;
const MARGIN: i32 = 24;

fn popup_size(scale: f64) -> (f64, f64) {
    (POPUP_WIDTH * scale, POPUP_HEIGHT * scale)
}

fn widget_size(scale: f64) -> (f64, f64) {
    (WIDGET_WIDTH * scale, WIDGET_HEIGHT * scale)
}

/// Esquina inferior derecha del monitor, con margen y hueco para la barra de tareas.
fn bottom_right_position(
    monitor_position: (i32, i32),
    monitor_size: (u32, u32),
    window_size: (u32, u32),
) -> (i32, i32) {
    let x = monitor_position.0 + monitor_size.0 as i32 - window_size.0 as i32 - MARGIN;
    let y = monitor_position.1 + monitor_size.1 as i32 - window_size.1 as i32 - MARGIN
        - TASKBAR_OFFSET;
    (x, y)
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Escala de interfaz configurada (1.0 = 100 %).
fn ui_scale(app: &AppHandle) -> f64 {
    app.try_state::<AppState>()
        .and_then(|state| {
            state
                .db
                .lock()
                .ok()
                .and_then(|conn| db::load_settings(&conn).ok())
        })
        .map(|settings| settings.font_scale as f64 / 100.0)
        .unwrap_or(1.0)
}

pub fn show_popup(app: &AppHandle) -> tauri::Result<()> {
    let (width, height) = popup_size(ui_scale(app));
    let window = ensure_window(app, "popup", width, height)?;
    let _ = window.set_size(LogicalSize::new(width, height));
    position_bottom_right(&window)?;

    if let Some(state) = app.try_state::<AppState>() {
        state.popup_visible.store(true, Ordering::Relaxed);
    }

    show_without_activation(&window)?;
    let _ = window.emit("challenge:show", ());
    Ok(())
}

pub fn show_widget(app: &AppHandle) -> tauri::Result<()> {
    let (width, height) = widget_size(ui_scale(app));
    let window = ensure_window(app, "widget", width, height)?;
    let _ = window.set_size(LogicalSize::new(width, height));
    position_bottom_right(&window)?;

    if let Some(state) = app.try_state::<AppState>() {
        state.widget_visible.store(true, Ordering::Relaxed);
    }

    show_without_activation(&window)?;
    let _ = window.emit("challenge:show", ());
    Ok(())
}

/// Redimensiona las ventanas flotantes cuando cambia la escala.
pub fn apply_font_scale(app: &AppHandle) {
    let scale = ui_scale(app);
    if let Some(window) = app.get_webview_window("popup") {
        let (width, height) = popup_size(scale);
        let _ = window.set_size(LogicalSize::new(width, height));
        let _ = position_bottom_right(&window);
    }
    if let Some(window) = app.get_webview_window("widget") {
        let (width, height) = widget_size(scale);
        let _ = window.set_size(LogicalSize::new(width, height));
        let _ = position_bottom_right(&window);
    }
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
    let language = app
        .try_state::<AppState>()
        .and_then(|state| state.db.lock().ok().and_then(|conn| db::load_settings(&conn).ok()))
        .map(|settings| settings.language)
        .unwrap_or_else(|| "es".to_string());
    show_notification(app, "kotoba", i18n::toast_body(&language));
}

pub fn show_notification(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;

    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
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

    crate::tray::sync_pause(app, settings.paused);
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
        let (x, y) = bottom_right_position(
            (monitor_position.x, monitor_position.y),
            (monitor_size.width, monitor_size.height),
            (size.width, size.height),
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_HD: (u32, u32) = (1920, 1080);

    #[test]
    fn scales_popup_size() {
        assert_eq!(popup_size(1.0), (392.0, 280.0));
        assert_eq!(popup_size(1.5), (588.0, 420.0));
        assert_eq!(popup_size(2.0), (784.0, 560.0));
    }

    #[test]
    fn scales_widget_size() {
        assert_eq!(widget_size(1.0), (340.0, 248.0));
        assert_eq!(widget_size(1.5), (510.0, 372.0));
        assert_eq!(widget_size(2.0), (680.0, 496.0));
    }

    #[test]
    fn positions_popup_at_bottom_right_with_taskbar_gap() {
        // 1920 - 392 - 24 = 1504; 1080 - 280 - 24 - 52 = 724
        assert_eq!(
            bottom_right_position((0, 0), FULL_HD, (392, 280)),
            (1504, 724)
        );
    }

    #[test]
    fn positions_widget_at_bottom_right_with_taskbar_gap() {
        // 1920 - 340 - 24 = 1556; 1080 - 248 - 24 - 52 = 756
        assert_eq!(
            bottom_right_position((0, 0), FULL_HD, (340, 248)),
            (1556, 756)
        );
    }

    #[test]
    fn positions_respect_scaled_window_size() {
        let (width, height) = popup_size(1.25);
        assert_eq!(
            bottom_right_position((0, 0), FULL_HD, (width as u32, height as u32)),
            (1406, 654)
        );
    }

    #[test]
    fn positions_on_secondary_monitor_use_monitor_offset() {
        // Monitor a la derecha del principal.
        assert_eq!(
            bottom_right_position((1920, 0), FULL_HD, (392, 280)),
            (3424, 724)
        );
        // Monitor a la izquierda: coordenadas negativas.
        assert_eq!(
            bottom_right_position((-1920, 0), FULL_HD, (392, 280)),
            (-416, 724)
        );
        // Monitor encima del principal.
        assert_eq!(
            bottom_right_position((0, -1080), FULL_HD, (392, 280)),
            (1504, -356)
        );
    }

    #[test]
    fn positions_handle_small_and_high_dpi_monitors() {
        // 1366x768: 1366 - 392 - 24 = 950; 768 - 280 - 24 - 52 = 412
        assert_eq!(
            bottom_right_position((0, 0), (1366, 768), (392, 280)),
            (950, 412)
        );
        // Monitor 4K: 3840 - 392 - 24 = 3424; 2160 - 280 - 24 - 52 = 1804
        assert_eq!(
            bottom_right_position((0, 0), (3840, 2160), (392, 280)),
            (3424, 1804)
        );
    }

    #[test]
    fn positions_can_be_negative_when_window_is_larger_than_monitor() {
        // No debe entrar en pánico ni saturar: la resta es con signo.
        assert_eq!(
            bottom_right_position((0, 0), (300, 200), (392, 280)),
            (-116, -156)
        );
    }
}
