fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "open_chat",
            "chat",
            "workspace_status",
            "setup_model_check",
            "suggest_wsl_profile",
            "setup_pick",
            "save_setup",
            "open_workspace_window",
            "workspace_projects",
            "prepare_app",
            "prepare_project",
            "decide_project",
            "tools_inventory",
            "inspect_rea",
        ]),
    ))
    .expect("AIDO desktop configuration could not be built");
}
