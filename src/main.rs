#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("desktop timer",
                       native_options,
                       Box::new(|cc| Box::new(desktop_timer::TemplateApp::new(cc)))).unwrap();
}
