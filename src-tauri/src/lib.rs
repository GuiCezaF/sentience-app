mod adapters;
mod agent;

use adapters::{HttpGateway, LiveAgent, LiveGateway};
use agent::{AgentInitError, DEFAULT_SUBJECT_ID, ModelPaths, StubGateway, TraySnapshot};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

static ALLOW_EXIT: AtomicBool = AtomicBool::new(false);
use tauri::{
    menu::{Menu, MenuItem},
    path::BaseDirectory,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    LogicalPosition, Manager, State, WebviewWindow,
};

#[tauri::command]
fn logout() -> String {
    "Logout".to_string()
}

#[tauri::command]
fn ingest_frame(bytes: Vec<u8>, agent: State<Mutex<LiveAgent>>) {
    let mut agent = agent.lock().expect("Agente");
    agent.ingest_frame(&bytes, SystemTime::now());
}

#[tauri::command]
fn set_camera_ok(ok: bool, agent: State<Mutex<LiveAgent>>) {
    agent.lock().expect("Agente").set_camera_ok(ok);
}

#[tauri::command]
fn snapshot(agent: State<Mutex<LiveAgent>>) -> TraySnapshot {
    agent.lock().expect("Agente").snapshot()
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    ALLOW_EXIT.store(true, Ordering::SeqCst);
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            logout,
            ingest_frame,
            set_camera_ok,
            snapshot,
            quit_app
        ])
        .setup(|app| {
            let agent = bootstrap_live_agent(app)?;
            app.manage(Mutex::new(agent));

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                handle
                    .state::<Mutex<LiveAgent>>()
                    .lock()
                    .expect("Agente")
                    .sync_tick(SystemTime::now());
                loop {
                    std::thread::sleep(Duration::from_secs(60));
                    handle
                        .state::<Mutex<LiveAgent>>()
                        .lock()
                        .expect("Agente")
                        .sync_tick(SystemTime::now());
                }
            });

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
                        ALLOW_EXIT.store(true, Ordering::SeqCst);
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if !ALLOW_EXIT.load(Ordering::SeqCst) {
                    api.prevent_exit();
                }
            }
        });
}

fn bootstrap_live_agent(app: &tauri::App) -> Result<LiveAgent, AgentInitError> {
    let resolver = app.path();
    let ultraface = resolver
        .resolve("model/version-RFB-320.onnx", BaseDirectory::Resource)
        .map_err(|e| {
            AgentInitError::new(format!(
                "Falha ao resolver model/version-RFB-320.onnx nos recursos do app: {e}"
            ))
        })?;
    let emotion = resolver
        .resolve("model/emotion_model.onnx", BaseDirectory::Resource)
        .map_err(|e| {
            AgentInitError::new(format!(
                "Falha ao resolver model/emotion_model.onnx nos recursos do app: {e}"
            ))
        })?;
    let app_data = resolver.app_data_dir().map_err(|e| {
        AgentInitError::new(format!("Falha ao resolver o diretório de dados do app: {e}"))
    })?;
    std::fs::create_dir_all(&app_data).map_err(|e| {
        AgentInitError::new(format!("Falha ao criar o diretório de dados do app: {e}"))
    })?;
    let queue_path = app_data.join("queue.sqlite");
    let subject_id =
        std::env::var("SENTIENCE_SUBJECT_ID").unwrap_or_else(|_| DEFAULT_SUBJECT_ID.to_string());
    LiveAgent::make(
        ModelPaths { ultraface, emotion },
        queue_path,
        subject_id,
        live_gateway_from_env(),
    )
}

fn live_gateway_from_env() -> LiveGateway {
    match std::env::var("SENTIENCE_SYNC_URL") {
        Ok(url) if !url.trim().is_empty() => LiveGateway::Http(HttpGateway::new(url)),
        _ => {
            let stub = if std::env::var("SENTIENCE_SYNC_FAIL").as_deref() == Ok("1") {
                StubGateway::failing()
            } else {
                StubGateway::ok()
            };
            LiveGateway::Stub(stub)
        }
    }
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

    let margin_right = 0.0;
    let margin_bottom = 14.0;

    let x = screen_width - width - margin_right;
    let y = screen_height - height - margin_bottom;

    window.set_position(LogicalPosition::new(x, y))?;

    window.show()?;
    window.unminimize()?;
    window.set_focus()?;

    Ok(())
}
