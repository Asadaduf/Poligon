use eframe::egui;
use egui::{RichText, TextureHandle, TextureOptions};
use image::GenericImageView;
use image::io::Reader as ImageReader;
use rand::Rng;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::io::BufReader;
use std::time::{Duration, Instant};

pub struct AdvancedApp {
    score: u32,
    enemies: Vec<Enemy>,
    supply_boxes: Vec<SupplyBox>,
    next_enemy_spawn_time: Instant,
    next_elite_spawn_time: Instant,
    next_supply_time: Instant,
    textures: HashMap<String, TextureHandle>,
    game_time: Duration,
    last_update: Instant,
    visible_time: i64,
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
    Alive {
        next_fire: Instant,
        #[allow(dead_code)]
        last_fired: Option<Instant>,
    },
    Firing {
        fire_start: Instant,
        #[allow(dead_code)]
        texture_key: String,
    },
    Dying(Instant),
}

enum EnemyType {
    Normal,
    Elite,
}
enum SupplyBoxType {
    Health,
    Tnt,
}

enum SupplyBoxState {
    Active,
    Damaged(Instant),
    Exploding(Instant),
    #[allow(dead_code)]
    Destroyed(Instant),
}

struct Enemy {
    x: f32,
    y: f32,
    texture_key: String,
    state: EnemyState,
    enemy_type: EnemyType,
    hitpoints: u32,
    #[allow(dead_code)]
    spawn_time: Instant,
}

struct SupplyBox {
    x: f32,
    y: f32,
    kind: SupplyBoxType,
    state: SupplyBoxState,
    spawn_time: Instant,
}

impl Enemy {
    pub fn new_elite() -> Self {
        let now = Instant::now();
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0..750.0);
        let y = rng.gen_range(0.0..500.0);
        let texture_key = format!("assets/sprite/elite-{}.png", rng.gen_range(1..=2));
        let next_fire = now + Duration::from_secs_f32(1.0);

        Enemy {
            x,
            y,
            texture_key,
            state: EnemyState::Alive {
                next_fire,
                last_fired: None,
            },
            enemy_type: EnemyType::Elite,
            hitpoints: 3,
            spawn_time: now,
        }
    }

    pub fn take_hit(&mut self) -> bool {
        if self.hitpoints > 1 {
            self.hitpoints -= 1;
            false
        } else {
            self.state = EnemyState::Dying(Instant::now());
            true
        }
    }
}

impl AdvancedApp {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let spawn_delay = Duration::from_secs_f32(rand::thread_rng().gen_range(0.5..=1.0));
        Self {
            score: 0,
            enemies: Vec::new(),
            supply_boxes: Vec::new(),
            next_enemy_spawn_time: Instant::now() + spawn_delay,
            next_elite_spawn_time: Instant::now() + Duration::from_secs(30),
            next_supply_time: Instant::now()
                + Duration::from_secs_f32(rand::thread_rng().gen_range(5.0..=8.0)),
            textures: HashMap::new(),
            game_time: Duration::ZERO,
            last_update: Instant::now(),
            visible_time: 30,
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

        let delta = now.duration_since(self.last_update);
        if !self.show_intro && !self.game_over {
            self.game_time += delta;
        }
        self.last_update = now;

        let mut signal = None;
        let mut fire_events: Vec<i64> = Vec::new();

        for name in [
            "assets/sprite/enemy-1.png",
            "assets/sprite/enemy-2.png",
            "assets/sprite/enemy_death-1.png",
            "assets/sprite/enemy_death-2.png",
            "assets/sprite/enemy_fire.png",
            "assets/sprite/supplybox_health.png",
            "assets/sprite/supplybox_tnt.png",
            "assets/sprite/supplybox_damaged.png",
            "assets/sprite/supplybox_explosion.png",
            "assets/sprite/supplybox_destroyed.png",
            "assets/sprite/elite-1.png",
            "assets/sprite/elite-2.png",
            "assets/sprite/elite_fire.png",
            "assets/sprite/elite_death-1.png",
            "assets/sprite/elite_death-2.png",
        ] {
            if !self.textures.contains_key(name) {
                let texture = load_image(ctx, name);
                self.textures.insert(name.to_string(), texture);
            }
        }

        let enemy_display_width = 75.0;
        let box_display_width = 60.0;
        let enemy_size = enemy_display_width * (55.0 / 35.0);
        let box_size = box_display_width * (35.0 / 35.0);

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

            if self.visible_time <= 0 {
                self.game_over = true;
            }

            if self.game_over {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.heading(RichText::new("Gelişmiş - Oyun Bitti!").size(32.0));
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

            ui.label(RichText::new(format!("Süre: {}", self.visible_time)).size(20.0));
            ui.label(RichText::new(format!("Skor: {}", self.score)).size(20.0));

            if now >= self.next_enemy_spawn_time {
                self.spawn_enemy();
                let spawn_delay = Duration::from_secs_f32(rand::thread_rng().gen_range(0.5..=1.0));
                self.next_enemy_spawn_time = now + spawn_delay;
            }

            if now >= self.next_elite_spawn_time && self.game_time >= Duration::from_secs(30) {
                self.spawn_elite();
            }

            let total_elapsed = now.duration_since(self.start_time);

            if now >= self.next_supply_time && total_elapsed >= Duration::from_secs(10) {
                let mut rng = rand::thread_rng();
                let kind = if rng.gen_bool(0.65) {
                    SupplyBoxType::Health
                } else {
                    SupplyBoxType::Tnt
                };
                let box_x = rng.gen_range(0.0..700.0);
                let box_y = rng.gen_range(0.0..450.0);
                self.supply_boxes.push(SupplyBox {
                    x: box_x,
                    y: box_y,
                    kind,
                    state: SupplyBoxState::Active,
                    spawn_time: now,
                });
                self.next_supply_time = now + Duration::from_secs_f32(rng.gen_range(5.0..=8.0));
            }

            self.enemies.retain_mut(|enemy| {
                match &mut enemy.state {
                    EnemyState::Alive { next_fire, .. } => {
                        if now >= *next_fire {
                            let damage = match enemy.enemy_type {
                                EnemyType::Normal => 1,
                                EnemyType::Elite => 3,
                            };
                            enemy.state = EnemyState::Firing {
                                fire_start: now,
                                texture_key: enemy.texture_key.clone(),
                            };
                            fire_events.push(damage);
                        }
                        true
                    }
                    EnemyState::Firing { fire_start, .. } => {
                        if now.duration_since(*fire_start) >= Duration::from_millis(500) {
                            let mut rng = rand::thread_rng();
                            let delay = rng.gen_range(0.7..=1.2);
                            enemy.state = EnemyState::Alive {
                                next_fire: now + Duration::from_secs_f32(delay),
                                last_fired: Some(*fire_start),
                            };
                        }
                        true
                    }
                    EnemyState::Dying(t0) => now.duration_since(*t0) < Duration::from_millis(500),
                }
            });

            self.supply_boxes.retain(|supply| match &supply.state {
                SupplyBoxState::Active => {
                    now.duration_since(supply.spawn_time) < Duration::from_secs(3)
                }
                SupplyBoxState::Destroyed(t)
                | SupplyBoxState::Damaged(t)
                | SupplyBoxState::Exploding(t) => {
                    now.duration_since(*t) < Duration::from_millis(500)
                }
            });

            for damage in fire_events {
                let sound = if damage == 3 {
                    "assets/sound/elite_fire.wav"
                } else {
                    "assets/sound/enemy_fire.mp3"
                };

                self.play_sound(sound);
                self.visible_time -= damage;
                self.visible_time = self.visible_time.clamp(0, 60);
            }

            for enemy in &self.enemies {
                let tex_key = match (&enemy.enemy_type, &enemy.state) {
                    (EnemyType::Normal, EnemyState::Alive { .. }) => enemy.texture_key.clone(),
                    (EnemyType::Normal, EnemyState::Firing { .. }) => {
                        "assets/sprite/enemy_fire.png".to_string()
                    }
                    (EnemyType::Normal, EnemyState::Dying(t0)) => {
                        if now.duration_since(*t0) < Duration::from_millis(250) {
                            "assets/sprite/enemy_death-1.png".to_string()
                        } else {
                            "assets/sprite/enemy_death-2.png".to_string()
                        }
                    }

                    (EnemyType::Elite, EnemyState::Alive { .. }) => enemy.texture_key.clone(),
                    (EnemyType::Elite, EnemyState::Firing { .. }) => {
                        "assets/sprite/elite_fire.png".to_string()
                    }
                    (EnemyType::Elite, EnemyState::Dying(t0)) => {
                        if now.duration_since(*t0) < Duration::from_millis(250) {
                            "assets/sprite/elite_death-1.png".to_string()
                        } else {
                            "assets/sprite/elite_death-2.png".to_string()
                        }
                    }
                };

                if let Some(texture) = self.textures.get(&tex_key) {
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(enemy.x, enemy.y),
                        egui::vec2(enemy_display_width, enemy_size),
                    );
                    ui.painter().image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            }

            for supply in &self.supply_boxes {
                let tex_key = match (&supply.kind, &supply.state) {
                    (SupplyBoxType::Health, SupplyBoxState::Active) => {
                        "assets/sprite/supplybox_health.png"
                    }
                    (SupplyBoxType::Health, SupplyBoxState::Damaged(t)) => {
                        if now.duration_since(*t) < Duration::from_millis(250) {
                            "assets/sprite/supplybox_damaged.png"
                        } else {
                            "assets/sprite/supplybox_destroyed.png"
                        }
                    }
                    (SupplyBoxType::Health, SupplyBoxState::Exploding(_)) => {
                        "assets/sprite/supplybox_destroyed.png"
                    }

                    (SupplyBoxType::Tnt, SupplyBoxState::Active) => {
                        "assets/sprite/supplybox_tnt.png"
                    }
                    (SupplyBoxType::Tnt, SupplyBoxState::Exploding(t)) => {
                        if now.duration_since(*t) < Duration::from_millis(250) {
                            "assets/sprite/supplybox_explosion.png"
                        } else {
                            "assets/sprite/supplybox_destroyed.png"
                        }
                    }
                    (SupplyBoxType::Tnt, SupplyBoxState::Damaged(_)) => {
                        "assets/sprite/supplybox_destroyed.png"
                    }

                    (_, SupplyBoxState::Destroyed(_)) => "assets/sprite/supplybox_destroyed.png",
                };

                if let Some(texture) = self.textures.get(tex_key) {
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(supply.x, supply.y),
                        egui::vec2(box_display_width, box_size),
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
            let mut clicked_pos = None;

            ctx.input(|i| {
                if i.pointer.primary_clicked() {
                    self.play_sound("assets/sound/gunshot.mp3");
                    clicked_pos = i.pointer.interact_pos();
                }
            });

            if let Some(pos) = clicked_pos {
                self.handle_click_on_enemy(pos.x, pos.y);

                for supply in &mut self.supply_boxes {
                    if let SupplyBoxState::Active = supply.state {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(supply.x, supply.y),
                            egui::vec2(box_display_width, box_size),
                        );
                        if rect.contains(pos) {
                            match supply.kind {
                                SupplyBoxType::Health => {
                                    supply.state = SupplyBoxState::Damaged(Instant::now());

                                    self.visible_time += 20;
                                    self.visible_time = self.visible_time.clamp(0, 60);

                                    let index = rand::thread_rng().gen_range(1..=3);
                                    let sound =
                                        format!("assets/sound/supplybox_damage-{}.mp3", index);
                                    self.play_sound(&sound);
                                }

                                SupplyBoxType::Tnt => {
                                    supply.state = SupplyBoxState::Exploding(Instant::now());

                                    self.visible_time -= 5;
                                    self.visible_time = self.visible_time.clamp(0, 60);

                                    let special = rand::thread_rng().gen_bool(0.01);
                                    let sound = if special {
                                        "assets/sound/supplybox_explosion-special.mp3".to_string()
                                    } else {
                                        "assets/sound/supplybox_explosion.mp3".to_string()
                                    };
                                    self.play_sound(&sound);

                                    self.explode_tnt();
                                }
                            }

                            break;
                        }
                    }
                }
            }

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

        let elapsed_secs = self.game_time.as_secs();
        let prev_elapsed_secs = self.game_time.saturating_sub(delta).as_secs();

        if elapsed_secs > prev_elapsed_secs {
            self.visible_time -= 1;
            self.visible_time = self.visible_time.clamp(0, 60);
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
        let next_fire = now + Duration::from_secs_f32(1.0);

        self.enemies.push(Enemy {
            x,
            y,
            texture_key,
            state: EnemyState::Alive {
                next_fire,
                last_fired: None,
            },
            enemy_type: EnemyType::Normal,
            hitpoints: 1,
            spawn_time: now,
        });
    }

    fn spawn_elite(&mut self) {
        let elite = Enemy::new_elite();
        self.enemies.push(elite);

        let delay = rand::thread_rng().gen_range(4.0..=8.0);
        self.next_elite_spawn_time = Instant::now() + Duration::from_secs_f32(delay);
    }

    fn explode_tnt(&mut self) {
        for enemy in &mut self.enemies {
            if let EnemyState::Alive { .. } | EnemyState::Firing { .. } = enemy.state {
                enemy.state = EnemyState::Dying(Instant::now());

                let bonus = match enemy.enemy_type {
                    EnemyType::Normal => 3,
                    EnemyType::Elite => 15,
                };
                self.score += bonus;
            }
        }
    }

    fn handle_click_on_enemy(&mut self, x: f32, y: f32) {
        for enemy in &mut self.enemies {
            let rect =
                egui::Rect::from_min_size(egui::pos2(enemy.x, enemy.y), egui::vec2(75.0, 117.85));
            if rect.contains(egui::pos2(x, y)) {
                match enemy.state {
                    EnemyState::Alive { .. } | EnemyState::Firing { .. } => {
                        let died = enemy.take_hit();
                        if died {
                            let bonus = match enemy.enemy_type {
                                EnemyType::Normal => 1,
                                EnemyType::Elite => 5,
                            };
                            self.score += bonus;

                            let mut rng = rand::thread_rng();
                            let chance: f64 = rng.r#gen();

                            match enemy.enemy_type {
                                EnemyType::Normal => {
                                    if chance < 0.05 {
                                        self.play_sound("assets/sound/enemy_death-special.mp3");
                                    } else {
                                        let index = rng.gen_range(1..=3);
                                        let path =
                                            format!("assets/sound/enemy_death-{}.wav", index);
                                        self.play_sound(&path);
                                    }
                                }
                                EnemyType::Elite => {
                                    if chance < 0.05 {
                                        self.play_sound("assets/sound/elite_death-special.mp3");
                                    } else {
                                        let index = rng.gen_range(1..=2);
                                        let path =
                                            format!("assets/sound/elite_death-{}.mp3", index);
                                        self.play_sound(&path);
                                    }
                                }
                            }
                        }
                        break;
                    }
                    _ => {}
                }
            }
        }
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
