#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aido_core::ai_client::ChatMessage;
use aido_core::companion_runtime::CompanionRuntime;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, PhysicalPosition, RunEvent, State, WebviewWindow, WindowEvent,
};

fn show_chat(app: &AppHandle) -> tauri::Result<()> {
    let chat = app
        .get_webview_window("chat")
        .ok_or(tauri::Error::WindowNotFound)?;
    chat.show()?;
    chat.unminimize()?;
    chat.set_focus()
}

#[tauri::command]
fn open_chat(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "mono" {
        return Err("Dieses Fenster kann den Chat nicht öffnen.".into());
    }
    show_chat(window.app_handle()).map_err(|error| error.to_string())
}

#[tauri::command]
async fn chat(
    window: WebviewWindow,
    runtime: State<'_, Arc<CompanionRuntime>>,
    messages: Vec<ChatMessage>,
) -> Result<String, String> {
    if window.label() != "chat" {
        return Err("Dieses Fenster hat keinen Zugriff auf das Modell.".into());
    }
    let runtime = Arc::clone(runtime.inner());
    tauri::async_runtime::spawn_blocking(move || runtime.chat(&messages))
        .await
        .map_err(|_| "Die lokale Modellanfrage konnte nicht ausgeführt werden.".to_owned())?
}

fn position_mono(window: &WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = window.primary_monitor()? {
        let area = monitor.work_area();
        let size = window.outer_size()?;
        let margin = (24.0 * monitor.scale_factor()).round() as i32;
        let x = (area.size.width as i32 - size.width as i32 - margin).max(0);
        let y = (area.size.height as i32 - size.height as i32 - margin).max(0);
        window.set_position(PhysicalPosition::new(
            area.position.x + x,
            area.position.y + y,
        ))?;
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(mono) = app.get_webview_window("mono") {
                let _ = mono.show();
            }
        }))
        .manage(Arc::new(CompanionRuntime::default()))
        .invoke_handler(tauri::generate_handler![open_chat, chat])
        .setup(|app| {
            let mono = app
                .get_webview_window("mono")
                .ok_or(tauri::Error::WindowNotFound)?;
            position_mono(&mono)?;

            let show = MenuItem::with_id(app, "show-mono", "MONO anzeigen", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide-mono", "MONO ausblenden", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "AIDO beenden", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("AIDO-MONO")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    } else if let Some(mono) = app.get_webview_window("mono") {
                        match event.id().as_ref() {
                            "show-mono" => {
                                let _ = mono.show();
                            }
                            "hide-mono" => {
                                let _ = mono.hide();
                            }
                            _ => {}
                        }
                    }
                });
            if let Some(icon) = app.default_window_icon().cloned() {
                tray = tray.icon(icon);
            }
            tray.build(app)?;
            mono.show()?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("AIDO konnte nicht initialisiert werden")
        .run(|app, event| {
            if matches!(event, RunEvent::Exit) {
                app.state::<Arc<CompanionRuntime>>().shutdown();
            }
        });
}
