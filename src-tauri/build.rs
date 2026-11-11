fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["open_chat", "chat"])),
    )
    .expect("AIDO desktop configuration could not be built");
}
