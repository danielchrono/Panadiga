fn main() {
    let _ = std::panic::catch_unwind(|| {
        let attrs = tauri_build::Attributes::new();
        let _ = tauri_build::try_build(attrs);
    });
}
