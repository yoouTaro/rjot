mod app;
mod capture;
mod doubletap;
mod exporter;
mod icons;
mod macos;
mod statusbar;
mod theme;

fn main() -> eframe::Result {
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("rjot")
        .with_inner_size([440.0, 660.0])
        .with_min_inner_size([420.0, 400.0])
        .with_fullsize_content_view(true)
        .with_titlebar_shown(false)
        .with_title_shown(false);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../../../assets/icon-512.png")) {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("rjot", options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}
