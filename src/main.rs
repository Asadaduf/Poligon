#![windows_subsystem = "windows"]

mod advanced;
mod classic;
mod menu;

use eframe::{App, Frame, NativeOptions, egui};
use egui::IconData;
use image::io::Reader as ImageReader;

enum Mode {
    Menu,
    Classic,
    Advanced,
    Wip(&'static str),
}

pub struct PoligonApp {
    mode: Mode,
    classic_state: Option<classic::ClassicApp>,
    advanced_state: Option<advanced::AdvancedApp>,
}

impl Default for PoligonApp {
    fn default() -> Self {
        Self {
            mode: Mode::Menu,
            classic_state: None,
            advanced_state: None,
        }
    }
}

impl App for PoligonApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        match &mut self.mode {
            Mode::Menu => {
                if let Some(result) = menu::draw_menu(ctx) {
                    match result.as_str() {
                        "classic" => {
                            self.mode = Mode::Classic;
                            self.classic_state = Some(classic::ClassicApp::new());
                        }
                        "advanced" => {
                            self.mode = Mode::Advanced;
                            self.advanced_state = Some(advanced::AdvancedApp::new());
                        }
                        "wip_bonus" => self.mode = Mode::Wip("Bonus mod çok yakında..."),
                        _ => {}
                    }
                }
            }
            Mode::Classic => {
                if let Some(app) = &mut self.classic_state {
                    if let Some(signal) = app.update(ctx, frame) {
                        match signal.as_str() {
                            "menu" => {
                                self.mode = Mode::Menu;
                                self.classic_state = None;
                            }
                            "restart" => {
                                *app = classic::ClassicApp::new();
                            }
                            _ => {}
                        }
                    }
                }
            }
            Mode::Advanced => {
                if let Some(app) = &mut self.advanced_state {
                    if let Some(signal) = app.update(ctx, frame) {
                        match signal.as_str() {
                            "menu" => {
                                self.mode = Mode::Menu;
                                self.advanced_state = None;
                            }
                            "restart" => {
                                *app = advanced::AdvancedApp::new();
                            }
                            _ => {}
                        }
                    }
                }
            }
            Mode::Wip(msg) => {
                if menu::draw_wip(ctx, msg) {
                    self.mode = Mode::Menu;
                }
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let icon_image = ImageReader::open("assets/sprite/icon.png")
        .expect("Simge dosyası açılamadı")
        .decode()
        .expect("Decode işlemi başarısız")
        .to_rgba8();

    let (width, height) = icon_image.dimensions();
    let rgba = icon_image.into_raw();

    let icon = IconData {
        rgba,
        width,
        height,
    };

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Poligon")
            .with_resizable(false)
            .with_maximize_button(false)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Poligon",
        options,
        Box::new(|_cc| Box::new(PoligonApp::default())),
    )
}
