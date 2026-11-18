use crate::db;
use crate::models::Settings;
use crate::{shell, AppState};
use chrono::Timelike;
use rand::Rng;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

/// Arranca el planificador pasivo. Revisa cada 10 s si toca mostrar un reto.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut next_run = Instant::now() + Duration::from_secs(75);

        loop {
            std::thread::sleep(Duration::from_secs(10));
            if Instant::now() < next_run {
                continue;
            }

            let state = app.state::<AppState>();
            let popup_visible = state.popup_visible.load(Ordering::Relaxed);
            let Ok(conn) = state.db.lock() else {
                next_run = Instant::now() + Duration::from_secs(60);
                continue;
            };
            let settings = db::load_settings(&conn).unwrap_or_default();
            let item_count = db::count_items(&conn).unwrap_or(0);
            let reviews_today = db::reviews_today(&conn).unwrap_or(0);
            drop(conn);

            if settings.mode == "popup" && popup_visible {
                next_run = Instant::now() + Duration::from_secs(45);
                continue;
            }

            let max_reached = settings.max_per_day > 0 && reviews_today >= settings.max_per_day;
            if item_count == 0 || settings.paused || in_dnd(&settings) || max_reached {
                next_run = Instant::now() + Duration::from_secs(60);
                continue;
            }

            if settings.suppress_fullscreen && shell::fullscreen_or_busy() {
                next_run = Instant::now() + Duration::from_secs(60);
                continue;
            }

            match settings.mode.as_str() {
                "widget" => {
                    let _ = shell::show_widget(&app);
                }
                "toast" => shell::show_toast(&app),
                _ => {
                    let _ = shell::show_popup(&app);
                }
            }

            let min = settings.interval_min_minutes.max(1) as u64;
            let max = settings.interval_max_minutes.max(settings.interval_min_minutes) as u64;
            let minutes = rand::thread_rng().gen_range(min..=max);
            next_run = Instant::now() + Duration::from_secs(minutes * 60);
        }
    });
}

fn in_dnd(settings: &Settings) -> bool {
    if !settings.dnd_enabled {
        return false;
    }
    let now = chrono::Local::now();
    let current = now.hour() * 60 + now.minute();
    let Some(start) = parse_hhmm(&settings.dnd_start) else {
        return false;
    };
    let Some(end) = parse_hhmm(&settings.dnd_end) else {
        return false;
    };
    if start == end {
        return false;
    }
    if start < end {
        current >= start && current < end
    } else {
        current >= start || current < end
    }
}

fn parse_hhmm(value: &str) -> Option<u32> {
    let mut parts = value.split(':');
    let hour = parts.next()?.parse::<u32>().ok()?;
    let minute = parts.next()?.parse::<u32>().ok()?;
    if hour >= 24 || minute >= 60 {
        return None;
    }
    Some(hour * 60 + minute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hhmm() {
        assert_eq!(parse_hhmm("22:00"), Some(1320));
        assert_eq!(parse_hhmm("08:30"), Some(510));
        assert_eq!(parse_hhmm("25:00"), None);
        assert_eq!(parse_hhmm("nope"), None);
    }
}
