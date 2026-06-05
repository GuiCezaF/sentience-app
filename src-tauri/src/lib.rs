use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    LogicalPosition, Manager, WebviewWindow,
};

#[tauri::command]
fn logout() -> String {
    return "Logout".to_string();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![logout])
        .setup(|app| {
            let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            let tray_icon = app
                .default_window_icon()
                .expect("Nenhum ícone encontrado no bundle")
                .clone();

            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let visible = window.is_visible().unwrap_or(false);

                            if visible {
                                let _ = window.hide();
                            } else {
                                let _ = show_tray_window(&window);
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Focused(false) => {
                let _ = window.hide();
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_tray_window(window: &WebviewWindow) -> tauri::Result<()> {
    let monitor = window
        .current_monitor()?
        .ok_or_else(|| tauri::Error::AssetNotFound("Monitor não encontrado".into()))?;

    let monitor_size = monitor.size();
    let scale = monitor.scale_factor();

    let window_size = window.outer_size()?;

    let width = window_size.width as f64 / scale;
    let height = window_size.height as f64 / scale;

    let screen_width = monitor_size.width as f64 / scale;
    let screen_height = monitor_size.height as f64 / scale;

    let margin_right = 10.0;
    let margin_bottom = 40.0;

    let x = screen_width - width - margin_right;
    let y = screen_height - height - margin_bottom;

    window.set_position(LogicalPosition::new(x, y))?;

    window.show()?;
    window.unminimize()?;
    window.set_focus()?;

    Ok(())
}
