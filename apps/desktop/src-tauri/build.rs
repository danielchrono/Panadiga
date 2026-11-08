fn main() {
    #[cfg(windows)]
    {
        let attrs = tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        let _ = tauri_build::try_build(attrs);
    }
    #[cfg(not(windows))]
    {
        let attrs = tauri_build::Attributes::new();
        let _ = tauri_build::try_build(attrs);
    }
}
