//! PaintCP — un semplice programma di disegno per macOS, scritto in Rust.

mod app;
mod paint;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("PaintCP")
            .with_app_id("PaintCP")
            .with_inner_size([1180.0, 820.0])
            .with_min_inner_size([760.0, 540.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "PaintCP",
        options,
        Box::new(|cc| Ok(Box::new(app::PaintApp::new(cc)))),
    )
}
