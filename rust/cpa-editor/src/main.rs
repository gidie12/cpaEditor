fn main() -> eframe::Result {
    eframe::run_simple_native("CPA Editor", eframe::NativeOptions::default(), |ctx, _frame| {
        eframe::egui::CentralPanel::default().show(ctx, |ui| ui.label("CPA Editor"));
    })
}
