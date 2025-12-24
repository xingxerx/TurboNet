mod gui;
use gui::MissionControlGui;

fn main() {
    dotenvy::dotenv().ok();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "TurboNet Mission Control",
        native_options,
        Box::new(|_cc| Ok(Box::new(MissionControlGui::default()))),
    ).unwrap(); // Ensure errors are surfaced
}


