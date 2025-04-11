use egui::{Color32, Context, RichText};
use once_cell::sync::OnceCell;
use std::time::{Duration, Instant};

static INTRO_START: OnceCell<Instant> = OnceCell::new();

pub fn draw_menu(ctx: &Context) -> Option<String> {
    let now = Instant::now();
    let mut result = None;

    INTRO_START.get_or_init(|| now);
    let start = *INTRO_START.get().unwrap();
    let elapsed = now.duration_since(start);

    if elapsed < Duration::from_secs(5) {
        let elapsed_secs = elapsed.as_secs_f32();
        let alpha = if elapsed_secs < 1.0 {
            elapsed_secs / 1.0
        } else if elapsed_secs < 4.0 {
            1.0
        } else {
            1.0 - ((elapsed_secs - 4.0) / 1.0)
        };

        let alpha_u8 = (alpha * 255.0).clamp(0.0, 255.0) as u8;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(200.0);
                ui.heading(
                    RichText::new("Poligona Hoşgeldin")
                        .size(32.0)
                        .color(Color32::from_rgba_unmultiplied(0, 0, 0, alpha_u8)),
                );
            });
        });

        ctx.request_repaint_after(Duration::from_millis(100));
        return None;
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading(RichText::new("Poligon").size(36.0));

            ui.add_space(40.0);

            ui.label(RichText::new("Geleneksel Poligon Deneyimi").size(20.0));
            ui.label(
                RichText::new("Old but gold")
                    .italics()
                    .size(16.0)
                    .color(Color32::GRAY),
            );
            let klasik_btn = ui.add_sized(
                [200.0, 40.0],
                egui::Button::new(RichText::new("Klasik").size(20.0)),
            );
            if klasik_btn.clicked() {
                result = Some("classic".to_string());
            }

            ui.add_space(30.0);

            ui.label(RichText::new("Geliştirilmiş Dinamik Poligon Mücadelesi").size(20.0));
            ui.label(
                RichText::new("Upgrades, people. Upgrades.")
                    .italics()
                    .size(16.0)
                    .color(Color32::GRAY),
            );
            let advanced_btn = ui.add_sized(
                [200.0, 40.0],
                egui::Button::new(RichText::new("Gelişmiş").size(20.0)),
            );
            if advanced_btn.clicked() {
                result = Some("advanced".to_string());
            }
        });
    });

    result
}

pub fn draw_wip(ctx: &Context, message: &str) -> bool {
    let mut back_to_menu = false;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(150.0);
            ui.heading(RichText::new(message).size(32.0));
            ui.add_space(20.0);
            if ui.button(RichText::new("Menüye Dön").size(24.0)).clicked() {
                back_to_menu = true;
            }
        });
    });

    back_to_menu
}
