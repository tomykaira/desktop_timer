#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![windows_subsystem = "windows"]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = native_options
        .viewport
        .with_transparent(true)
        // According to your preference
        // .with_decorations(false)
        .with_always_on_top();
    native_options.persist_window = false;
    // Prevent the window from disappearing in a multi-display environment.
    native_options.centered = true;
    eframe::run_native(
        "desktop timer",
        native_options,
        Box::new(|cc| Box::new(desktop_timer::TemplateApp::new(cc))),
    )
    .unwrap();
}
