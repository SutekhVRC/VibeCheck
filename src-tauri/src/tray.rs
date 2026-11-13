/*
 * The system tray icon and its menu.
 *
 *   VibeCheck 0.6.2               (header, greyed out)
 *   On · 2 toys connected         (status, greyed out)
 *   ---
 *   Stop all toys / Resume toys   (panic stop, same as Ctrl+Alt+P)
 *   Turn VibeCheck on / off
 *   Look for toys
 *   ---
 *   Show window / Hide window
 *   ---
 *   Restart
 *   Quit VibeCheck
 *
 * The menu is drawn by the operating system, so it can't be themed. Turning
 * on or off and scanning go through the frontend, as the OSC commands do
 * (FeStateEvent), so the app's own state stays in step.
 *
 * The changing lines are refreshed after every tray action, and checked once
 * a second for changes made anywhere else (sidebar, OSC, toys connecting).
 * The check never waits for the state lock: if it's busy, it tries again on
 * the next pass.
 */

use std::time::Duration;

use log::warn;
use parking_lot::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

use crate::{
    frontend::frontend_types::{FeCoreEvent, FeStateEvent},
    vcore::{
        ipc::emit_plane::emit_core_event,
        panic,
        state::{RunningState, VCStateMutex},
    },
};

const REFRESH_EVERY: Duration = Duration::from_secs(1);

/// What the changing lines show
#[derive(Clone, Copy, PartialEq, Eq)]
struct Status {
    running: bool,
    panicked: bool,
    toys: usize,
    window_visible: bool,
}

/// The menu items whose text changes
struct TrayMenu {
    status: MenuItem<Wry>,
    panic: MenuItem<Wry>,
    power: MenuItem<Wry>,
    window: MenuItem<Wry>,
    shown: Mutex<Option<Status>>,
}

pub fn build(app: &tauri::App) -> tauri::Result<()> {
    let version = app.package_info().version.to_string();
    let header = MenuItem::with_id(
        app,
        "header",
        format!("VibeCheck {version}"),
        false,
        None::<&str>,
    )?;
    let status = MenuItem::with_id(app, "status", "Off", false, None::<&str>)?;
    let panic_item = MenuItem::with_id(app, "panic", "Stop all toys", true, None::<&str>)?;
    let power = MenuItem::with_id(app, "power", "Turn VibeCheck on", true, None::<&str>)?;
    let scan = MenuItem::with_id(app, "scan", "Look for toys", true, None::<&str>)?;
    let window = MenuItem::with_id(app, "window", "Hide window", true, None::<&str>)?;
    let restart = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit VibeCheck", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &header,
            &status,
            &PredefinedMenuItem::separator(app)?,
            &panic_item,
            &power,
            &scan,
            &PredefinedMenuItem::separator(app)?,
            &window,
            &PredefinedMenuItem::separator(app)?,
            &restart,
            &quit,
        ],
    )?;

    app.manage(TrayMenu {
        status,
        panic: panic_item,
        power,
        window,
        shown: Mutex::new(None),
    });

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("VibeCheck")
        // A simpler version of the app icon that stays readable at tray size
        // (icons/source/app-icon-small.svg)
        .icon(tauri::include_image!("icons/tray.png"))
        // Windows: left click opens the window, right click the menu. Linux
        // trays always open the menu.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
                refresh(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "panic" => {
                    if panic::is_panicked(app) {
                        panic::resume_from_panic(app);
                    } else {
                        panic::trigger_panic(app);
                    }
                }
                "power" => {
                    let event = if is_running(app) {
                        FeStateEvent::Disable
                    } else {
                        FeStateEvent::Enable
                    };
                    emit_core_event(app, FeCoreEvent::State(event));
                }
                "scan" => {
                    let event = if is_running(app) {
                        FeStateEvent::Scan
                    } else {
                        FeStateEvent::EnableAndScan
                    };
                    emit_core_event(app, FeCoreEvent::State(event));
                }
                "window" => match app.get_webview_window("main") {
                    Some(window) if window.is_visible().unwrap_or(false) => {
                        let _ = window.hide();
                    }
                    _ => show_window(app),
                },
                "restart" => app.restart(),
                "quit" => app.exit(0),
                _ => {}
            }
            refresh(app);
        })
        .build(app)?;

    let app_handle = app.handle().clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(REFRESH_EVERY);
        refresh(&app_handle);
    });

    refresh(app.handle());
    Ok(())
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn is_running(app: &AppHandle) -> bool {
    matches!(
        app.state::<VCStateMutex>().0.lock().running,
        RunningState::Running
    )
}

/// Updates the changing lines if anything changed. Skips this pass if the
/// state is busy.
fn refresh(app: &AppHandle) {
    let Some(tray) = app.try_state::<TrayMenu>() else {
        return;
    };
    let status = {
        let state = app.state::<VCStateMutex>();
        let Some(vc_lock) = state.0.try_lock() else {
            return;
        };
        Status {
            running: matches!(vc_lock.running, RunningState::Running),
            panicked: vc_lock.safety.is_panicked(),
            toys: vc_lock
                .core_toy_manager
                .as_ref()
                .map_or(0, |m| m.online_toys.len()),
            window_visible: app
                .get_webview_window("main")
                .and_then(|w| w.is_visible().ok())
                .unwrap_or(false),
        }
    };

    let mut shown = tray.shown.lock();
    if *shown == Some(status) {
        return;
    }
    *shown = Some(status);
    drop(shown);

    let set = |item: &MenuItem<Wry>, text: &str| {
        if let Err(e) = item.set_text(text) {
            warn!("Couldn't update the tray menu: {}", e);
        }
    };
    set(&tray.status, &status_line(status));
    set(
        &tray.panic,
        if status.panicked {
            "Resume toys"
        } else {
            "Stop all toys"
        },
    );
    set(
        &tray.power,
        if status.running {
            "Turn VibeCheck off"
        } else {
            "Turn VibeCheck on"
        },
    );
    set(
        &tray.window,
        if status.window_visible {
            "Hide window"
        } else {
            "Show window"
        },
    );
}

fn status_line(status: Status) -> String {
    if status.panicked {
        return "All toys stopped".to_owned();
    }
    if !status.running {
        return "Off".to_owned();
    }
    match status.toys {
        0 => "On \u{b7} No toys connected".to_owned(),
        1 => "On \u{b7} 1 toy connected".to_owned(),
        n => format!("On \u{b7} {n} toys connected"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_reads_naturally() {
        let status = |running, panicked, toys| Status {
            running,
            panicked,
            toys,
            window_visible: true,
        };
        assert_eq!(status_line(status(false, false, 0)), "Off");
        assert_eq!(
            status_line(status(true, false, 0)),
            "On \u{b7} No toys connected"
        );
        assert_eq!(
            status_line(status(true, false, 1)),
            "On \u{b7} 1 toy connected"
        );
        assert_eq!(
            status_line(status(true, false, 3)),
            "On \u{b7} 3 toys connected"
        );
        assert_eq!(status_line(status(true, true, 3)), "All toys stopped");
    }
}
