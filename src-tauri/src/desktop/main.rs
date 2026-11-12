#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod workspace;

use aido_core::ai_client::ChatMessage;
use aido_core::companion_runtime::CompanionRuntime;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, PhysicalPosition, RunEvent, State, WebviewWindow, WindowEvent,
};
use workspace::{
    approve_proposal, blender_inspect, capture_desktop_screen, decide_project, docker_containers,
    docker_status, get_active_proposals, get_telemetry_snapshot, inspect_rea,
    launch_project_orchestrated, memory_forget, memory_overview, memory_search,
    open_workspace_window, prepare_app, prepare_project, proactive_check, proactive_dismiss,
    reject_proposal, save_setup, scan_projects_catalog, seed_demo_state, setup_model_check,
    setup_pick, suggest_wsl_profile, system_security_status, tools_inventory, unload_model_vram,
    workspace_projects, workspace_status, Workspace,
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
    workspace: State<'_, Arc<Workspace>>,
    mut messages: Vec<ChatMessage>,
) -> Result<String, String> {
    if window.label() != "chat" {
        return Err("Dieses Fenster hat keinen Zugriff auf das Modell.".into());
    }
    let profile = workspace
        .store
        .active()?
        .ok_or("Schliesse zuerst AIDOs Einrichtung ab.")?
        .model_profile;

    // Check for "Vergiss ..." command
    if let Some(last_msg) = messages.iter().rev().find(|m| m.role == aido_core::ai_client::ChatRole::User) {
        let text = last_msg.content.trim();
        let lower = text.to_lowercase();
        if lower.starts_with("vergiss ") || lower == "vergiss das" {
            let term = if lower == "vergiss das" { "" } else { text[8..].trim() };
            let ws = Arc::clone(workspace.inner());
            let found = if term.is_empty() {
                ws.memory.list_recent_memories(1).unwrap_or_default()
            } else {
                ws.memory.search_memory(term, 3).unwrap_or_default()
            };
            if let Some(target) = found.first() {
                let _ = ws.memory.forget_memory_entry(target.id);
                return Ok(format!("Ich habe den Eintrag '{}' aus meinem aktiven Gedächtnis gelöscht.", target.summary));
            } else {
                return Ok("Ich habe keinen passenden Eintrag im Gedächtnis gefunden, den ich vergessen könnte.".to_string());
            }
        }
    }

    // Memory Context Enrichment
    if let Some(last_msg) = messages.iter_mut().rev().find(|m| m.role == aido_core::ai_client::ChatRole::User) {
        let ws = Arc::clone(workspace.inner());
        let hits = ws.memory.search_memory(&last_msg.content, 3).unwrap_or_default();
        if !hits.is_empty() {
            let memory_context = hits
                .iter()
                .map(|h| format!("- [{}]: {} ({})", h.layer, h.summary, h.details))
                .collect::<Vec<_>>()
                .join("\n");
            last_msg.content = format!(
                "[AIDO Gedächtnis-Kontext]:\n{}\n\n{}",
                memory_context, last_msg.content
            );
        }
    }

    let user_input_to_learn = messages
        .iter()
        .rev()
        .find(|m| m.role == aido_core::ai_client::ChatRole::User)
        .map(|m| m.content.clone());

    let runtime = Arc::clone(runtime.inner());
    let response = tauri::async_runtime::spawn_blocking(move || runtime.chat_with_profile(&messages, &profile))
        .await
        .map_err(|_| "Die lokale Modellanfrage konnte nicht ausgeführt werden.".to_owned())??;

    // Automatic Learning of User Preferences & Facts
    if let Some(ref text) = user_input_to_learn {
        let lower = text.to_lowercase();
        if lower.contains("mein lieblings")
            || lower.contains("ich bevorzuge")
            || lower.contains("merke dir")
            || lower.contains("mein projekt")
            || lower.contains("ich heiße")
            || lower.contains("ich heisse")
        {
            let ws = Arc::clone(workspace.inner());
            let _ = ws.memory.store_memory(
                aido_core::memory::MemoryLayer::UserMemory.as_str(),
                "preferences",
                "Benutzerpräferenz",
                text,
                "chat",
            );
        }
    }

    Ok(response)
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
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::StationsAndDesktops::{
            GetThreadDesktop, GetUserObjectInformationW, OpenDesktopW, SetThreadDesktop, UOI_NAME,
        };
        unsafe {
            let thread_id = windows_sys::Win32::System::Threading::GetCurrentThreadId();
            let desktop = GetThreadDesktop(thread_id);
            let mut name_buf = [0u16; 256];
            let mut length = 0;
            if GetUserObjectInformationW(
                desktop as _,
                UOI_NAME as _,
                name_buf.as_mut_ptr() as _,
                512,
                &mut length,
            ) != 0
            {
                let name = String::from_utf16_lossy(&name_buf[..(length as usize / 2).saturating_sub(1)]);
                if !name.eq_ignore_ascii_case("default") {
                    let default_desktop = OpenDesktopW(
                        windows_sys::core::w!("Default"),
                        0,
                        0,
                        0x01FF,
                    );
                    if !default_desktop.is_null() {
                        let _ = SetThreadDesktop(default_desktop);
                    }
                }
            }
        }
    }

    let root = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .expect("LOCALAPPDATA fehlt.")
        .join("AIDO");
    let workspace =
        Arc::new(Workspace::new(&root).expect("AIDOs Datenbank konnte nicht geöffnet werden."));
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let active = app
                .try_state::<Arc<Workspace>>()
                .is_some_and(|state| state.store.active().ok().flatten().is_some());
            if !active {
                let _ = workspace::show(app, "setup");
            } else if let Some(mono) = app.get_webview_window("mono") {
                let _ = mono.show();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(CompanionRuntime::default()))
        .manage(workspace)
        .invoke_handler(tauri::generate_handler![
            open_chat,
            chat,
            workspace_status,
            setup_model_check,
            suggest_wsl_profile,
            setup_pick,
            save_setup,
            open_workspace_window,
            workspace_projects,
            prepare_app,
            prepare_project,
            decide_project,
            tools_inventory,
            inspect_rea,
            memory_overview,
            memory_forget,
            memory_search,
            proactive_check,
            proactive_dismiss,
            blender_inspect,
            docker_status,
            docker_containers,
            system_security_status,
            get_active_proposals,
            approve_proposal,
            reject_proposal,
            get_telemetry_snapshot,
            unload_model_vram,
            scan_projects_catalog,
            launch_project_orchestrated,
            capture_desktop_screen,
            seed_demo_state,
        ])
        .setup(|app| {
            let configured = app
                .state::<Arc<Workspace>>()
                .store
                .active()
                .map_err(std::io::Error::other)?
                .is_some();
            let mono = app
                .get_webview_window("mono")
                .ok_or(tauri::Error::WindowNotFound)?;
            position_mono(&mono)?;

            let show = MenuItem::with_id(app, "show-mono", "MONO anzeigen", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide-mono", "MONO ausblenden", true, None::<&str>)?;
            let memory_item = MenuItem::with_id(app, "show-memory", "🧠 Gedächtnis & Wissen", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "AIDO beenden", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &memory_item, &quit])?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("AIDO-MONO")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    } else if event.id().as_ref() == "show-memory" {
                        let _ = workspace::show(app, "memory");
                    } else if let Some(mono) = app.get_webview_window("mono") {
                        match event.id().as_ref() {
                            "show-mono" => {
                                let active = app
                                    .state::<Arc<Workspace>>()
                                    .store
                                    .active()
                                    .ok()
                                    .flatten()
                                    .is_some();
                                if active {
                                    let _ = mono.show();
                                } else {
                                    let _ = workspace::show(app, "setup");
                                }
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

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    if let Some(ws) = handle.try_state::<Arc<Workspace>>() {
                        let snap = ws.get_telemetry_snapshot_internal();
                        let _ = handle.emit("telemetry://update", &snap);
                    }
                }
            });
            if configured {
                mono.show()?;
            } else {
                workspace::show(app.handle(), "setup").map_err(std::io::Error::other)?;
            }
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
