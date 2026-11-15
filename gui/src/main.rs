#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod backend;
mod models;
mod views;

use app::LarpFlowApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LarpFlow - Developer Log Studio")
            .with_inner_size([1120.0, 780.0])
            .with_min_inner_size([860.0, 580.0]),
        ..Default::default()
    };

    eframe::run_native(
        "LarpFlow",
        native_options,
        Box::new(|_cc| Ok(Box::new(LarpFlowApp::default()))),
    )
}
