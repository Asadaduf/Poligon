// classic.rs
use eframe::egui;
use egui::{RichText, TextureHandle, TextureOptions};
use image::GenericImageView;
use image::io::Reader as ImageReader;
use rand::Rng;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::io::BufReader;
use std::time::{Duration, Instant};

pub struct ClassicApp {
    score: u32,
    enemies: Vec<Enemy>,
    next_spawn_time: Instant,
    textures: HashMap<String, TextureHandle>,
    start_time: Instant,
    game_over: bool,
    show_intro: bool,
    intro_start: Instant,
    restart_count: u32,
    #[allow(dead_code)]
    audio_stream: OutputStream,
    audio_handle: OutputStreamHandle,
}

enum EnemyState {
    Alive,
    Dying(Instant),
}

struct Enemy {
    x: f32,
    y: f32,
    texture_key: String,
    state: EnemyState,
    spawn_time: Instant,
}

impl ClassicApp {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let spawn_delay = Duration::from_secs_f32(rand::thread_rng().gen_range(0.5..=1.0));
        Self {
            score: 0,
            enemies: Vec::new(),
            next_spawn_time: Instant::now() + spawn_delay,
            textures: HashMap::new(),
            start_time: Instant::now(),
            game_over: false,
            show_intro: true,
            intro_start: Instant::now(),
            restart_count: 0,
            audio_stream: stream,
            audio_handle: stream_handle,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<String> {
        let now = Instant::now();
        let mut signal = None;

        for name in [
            "assets/sprite/enemy-1.png",
            "assets/sprite/enemy-2.png",
            "assets/sprite/enemy_death-1.png",
            "assets/sprite/enemy_death-2.png",
        ] {
            if !self.textures.contains_key(name) {
                let texture = load_image(ctx, name);
                self.textures.insert(name.to_string(), texture);
            }
        }

        let display_width = 75.0;
        let display_height = display_width * (55.0 / 35.0);

        egui::CentralPanel::default().show(ctx, |ui| {
            let elapsed_intro = now.duration_since(self.intro_start);

            if self.show_intro {
                ui.vertical_centered(|ui| {
                    ui.add_space(200.0);
                    if self.restart_count == 0 {
                        if elapsed_intro < Duration::from_secs(2) {
                            ui.heading(RichText::new("Hazır mısın?").size(36.0));
                        } else if elapsed_intro < Duration::from_secs(4) {
                            ui.heading(RichText::new("Başla!").size(36.0));
                        } else {
                            self.start_time = now;
                            self.show_intro = false;
                        }
                    } else {
                        if elapsed_intro < Duration::from_secs(2) {
                            ui.heading(RichText::new("Hazır mısın?").size(36.0));
                        } else if elapsed_intro < Duration::from_secs(4) {
                            ui.heading(RichText::new("Başla!").size(36.0));
                        } else {
                            self.start_time = now;
                            self.show_intro = false;
                        }
                    }
                });
                return;
            }

            let elapsed = now.duration_since(self.start_time);
            if elapsed >= Duration::from_secs(20) {
                self.game_over = true;
            }

            if self.game_over {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.heading(RichText::new("Oyun Bitti!").size(32.0));
                    ui.add_space(20.0);
                    ui.label(RichText::new(format!("Toplam Puan: {}", self.score)).size(24.0));
                    ui.add_space(20.0);

                    if ui.button(RichText::new("Tekrar Oyna").size(20.0)).clicked() {
                        signal = Some("restart".to_string());
                    }

                    ui.add_space(10.0);

                    if ui.button(RichText::new("Menüye Dön").size(20.0)).clicked() {
                        signal = Some("menu".to_string());
                    }
                });
                return;
            }

            ui.label(RichText::new(format!("Süre: {}", 30 - elapsed.as_secs().min(30))).size(20.0));
            ui.label(RichText::new(format!("Puan: {}", self.score)).size(20.0));

            if now >= self.next_spawn_time {
                self.spawn_enemy();
                let spawn_delay = Duration::from_secs_f32(rand::thread_rng().gen_range(0.5..=1.0));
                self.next_spawn_time = now + spawn_delay;
            }

            self.enemies.retain_mut(|enemy| match &mut enemy.state {
                EnemyState::Alive => now.duration_since(enemy.spawn_time) < Duration::from_secs(3),
                EnemyState::Dying(t0) => now.duration_since(*t0) < Duration::from_millis(500),
            });

            for enemy in &self.enemies {
                let tex_key = match &enemy.state {
                    EnemyState::Alive => enemy.texture_key.clone(),
                    EnemyState::Dying(t0) => {
                        if now.duration_since(*t0) < Duration::from_millis(250) {
                            "assets/sprite/enemy_death-1.png".to_string()
                        } else {
                            "assets/sprite/enemy_death-2.png".to_string()
                        }
                    }
                };
                if let Some(texture) = self.textures.get(&tex_key) {
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(enemy.x, enemy.y),
                        egui::vec2(display_width, display_height),
                    );
                    ui.painter().image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            }
        });

        if !self.show_intro && !self.game_over {
            ctx.input(|i| {
                if i.pointer.primary_clicked() {
                    self.play_sound("assets/sound/gunshot.mp3");
                    if let Some(pos) = i.pointer.interact_pos() {
                        for enemy in &mut self.enemies {
                            if matches!(enemy.state, EnemyState::Alive) || matches!(enemy.state, EnemyState::Dying(_)){
                                let rect = egui::Rect::from_min_size(
                                    egui::pos2(enemy.x, enemy.y),
                                    egui::vec2(display_width, display_height),
                                );
                                if rect.contains(pos) {
                                    if let EnemyState::Alive = enemy.state {
                                        enemy.state = EnemyState::Dying(Instant::now());
                                    }

                                    self.score += 1;

                                    let mut rng = rand::thread_rng();
                                    let chance: f64 = rng.r#gen();
                                    if chance < 0.05 {
                                        self.play_sound("assets/sound/enemy_death-special.mp3");
                                    } else {
                                        let index = rng.gen_range(1..=3);
                                        let death_file =
                                            format!("assets/sound/enemy_death-{}.wav", index);
                                        self.play_sound(&death_file);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            });

            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                let painter = ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("crosshair"),
                ));
                let size = 15.0;
                let gap = 5.0;
                let color = egui::Color32::RED;
                painter.line_segment(
                    [
                        pos - egui::vec2(size + gap, 0.0),
                        pos - egui::vec2(gap, 0.0),
                    ],
                    (2.0, color),
                );
                painter.line_segment(
                    [
                        pos + egui::vec2(gap, 0.0),
                        pos + egui::vec2(size + gap, 0.0),
                    ],
                    (2.0, color),
                );
                painter.line_segment(
                    [
                        pos - egui::vec2(0.0, size + gap),
                        pos - egui::vec2(0.0, gap),
                    ],
                    (2.0, color),
                );
                painter.line_segment(
                    [
                        pos + egui::vec2(0.0, gap),
                        pos + egui::vec2(0.0, size + gap),
                    ],
                    (2.0, color),
                );
            }
        }

        ctx.request_repaint_after(Duration::from_millis(100));
        signal
    }

    fn spawn_enemy(&mut self) {
        let now = Instant::now();
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0..750.0);
        let y = rng.gen_range(0.0..500.0);
        let texture_key = format!("assets/sprite/enemy-{}.png", rng.gen_range(1..=2));
        self.enemies.push(Enemy {
            x,
            y,
            texture_key,
            state: EnemyState::Alive,
            spawn_time: now,
        });
    }

    fn play_sound(&self, path: &str) {
        if let Ok(file) = std::fs::File::open(path) {
            let buffered = BufReader::new(file);
            if let Ok(source) = Decoder::new(buffered) {
                if let Ok(sink) = Sink::try_new(&self.audio_handle) {
                    sink.append(source);
                    sink.detach();
                }
            }
        }
    }
}

fn load_image(ctx: &egui::Context, path: &str) -> TextureHandle {
    let img = ImageReader::open(path).unwrap().decode().unwrap();
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();
    let color_image =
        egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &rgba);
    ctx.load_texture(path, color_image, TextureOptions::default())
}
