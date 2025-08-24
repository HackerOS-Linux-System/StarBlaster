use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use ::rand::thread_rng;
use ::rand::Rng;

// Serializable Vec2
#[derive(Serialize, Deserialize, Copy, Clone)]
struct SerVec2 {
    x: f32,
    y: f32,
}

// Serializable Enemy
#[derive(Serialize, Deserialize)]
struct SerEnemy {
    pos: SerVec2,
    speed: f32,
    alive: bool,
    enemy_type: u8, // 0: normal, 1: fast, 2: tank
}

// Serializable Bullet
#[derive(Serialize, Deserialize)]
struct SerBullet {
    pos: SerVec2,
    speed: f32,
    alive: bool,
    bullet_type: u8, // 0: normal, 1: power-up
}

// Serializable PowerUp
#[derive(Serialize, Deserialize)]
struct SerPowerUp {
    pos: SerVec2,
    speed: f32,
    alive: bool,
    power_type: u8, // 0: health, 1: double shot
}

// Struktura zapisu gry
#[derive(Serialize, Deserialize)]
struct GameSave {
    score: i32,
    player_pos: SerVec2,
    difficulty: f32,
    enemies: Vec<SerEnemy>,
    bullets: Vec<SerBullet>,
    power_ups: Vec<SerPowerUp>,
    player_health: i32,
    level: u32,
    double_shot: bool,
}

// Struktura ustawień
#[derive(Serialize, Deserialize)]
struct Settings {
    difficulty: f32,
    sound_volume: f32,
    high_score: i32,
}

// Stany gry
#[derive(PartialEq)]
enum GameState {
    Menu,
    Game,
    Settings,
    LoadGame,
}

// Struktura wroga
struct Enemy {
    pos: Vec2,
    speed: f32,
    alive: bool,
    enemy_type: u8, // 0: normal, 1: fast, 2: tank
    health: i32,    // Dla tanków więcej zdrowia
}

// Struktura pocisku
struct Bullet {
    pos: Vec2,
    speed: f32,
    alive: bool,
    bullet_type: u8, // 0: normal, 1: power-up (silniejszy)
}

// Struktura power-up
struct PowerUp {
    pos: Vec2,
    speed: f32,
    alive: bool,
    power_type: u8, // 0: health, 1: double shot
}

#[macroquad::main("StarBlaster")]
async fn main() {
    // Wczytaj ustawienia
    let mut settings = load_settings();

    // Inicjalizacja zmiennych gry
    let mut game_state = GameState::Menu;
    let mut player_pos = vec2(screen_width() / 2.0, screen_height() - 50.0);
    let mut player_health = 3;
    let mut score = 0;
    let mut enemies: Vec<Enemy> = vec![];
    let mut bullets: Vec<Bullet> = vec![];
    let mut power_ups: Vec<PowerUp> = vec![];
    let mut last_shot = get_time();
    let mut level = 1u32;
    let mut double_shot = false;
    let mut last_power_up = get_time();

    loop {
        clear_background(BLACK);

        match game_state {
            GameState::Menu => {
                // Menu główne
                draw_text("StarBlaster", screen_width() / 2.0 - 100.0, 100.0, 40.0, WHITE);
                draw_text(&format!("High Score: {}", settings.high_score), screen_width() / 2.0 - 100.0, 150.0, 30.0, WHITE);
                if draw_button("Start", screen_width() / 2.0 - 50.0, 200.0) {
                    game_state = GameState::Game;
                    score = 0;
                    player_health = 3;
                    enemies.clear();
                    bullets.clear();
                    power_ups.clear();
                    player_pos = vec2(screen_width() / 2.0, screen_height() - 50.0);
                    level = 1;
                    double_shot = false;
                }
                if draw_button("Load Game", screen_width() / 2.0 - 50.0, 250.0) {
                    game_state = GameState::LoadGame;
                }
                if draw_button("Settings", screen_width() / 2.0 - 50.0, 300.0) {
                    game_state = GameState::Settings;
                }
                if draw_button("Exit", screen_width() / 2.0 - 50.0, 350.0) {
                    break;
                }
            }
            GameState::Game => {
                // Logika gry
                // Sterowanie graczem
                if is_key_down(KeyCode::Left) {
                    player_pos.x -= 300.0 * get_frame_time();
                }
                if is_key_down(KeyCode::Right) {
                    player_pos.x += 300.0 * get_frame_time();
                }
                if is_key_down(KeyCode::Up) {
                    player_pos.y -= 300.0 * get_frame_time();
                }
                if is_key_down(KeyCode::Down) {
                    player_pos.y += 300.0 * get_frame_time();
                }
                // Ograniczenie ruchu gracza
                player_pos.x = player_pos.x.clamp(0.0, screen_width() - 20.0);
                player_pos.y = player_pos.y.clamp(0.0, screen_height() - 20.0);

                // Strzelanie
                if is_key_down(KeyCode::Space) && get_time() - last_shot > 0.2 {
                    bullets.push(Bullet {
                        pos: vec2(player_pos.x + 10.0, player_pos.y),
                                 speed: -400.0,
                                 alive: true,
                                 bullet_type: if double_shot { 1 } else { 0 },
                    });
                    if double_shot {
                        bullets.push(Bullet {
                            pos: vec2(player_pos.x - 10.0, player_pos.y),
                                     speed: -400.0,
                                     alive: true,
                                     bullet_type: 1,
                        });
                    }
                    last_shot = get_time();
                }

                // Spawn wrogów
                let spawn_chance = 0.02 * settings.difficulty * (level as f32 / 10.0 + 1.0);
                if thread_rng().gen_range(0.0..1.0) < spawn_chance {
                    let enemy_type = thread_rng().gen_range(0..3);
                    let speed = match enemy_type {
                        1 => 150.0 * settings.difficulty,
                        _ => 100.0 * settings.difficulty,
                    };
                    let health = match enemy_type {
                        2 => 3,
                        _ => 1,
                    };
                    enemies.push(Enemy {
                        pos: vec2(thread_rng().gen_range(0.0..screen_width()), 0.0),
                                 speed,
                                 alive: true,
                                 enemy_type,
                                 health,
                    });
                }

                // Spawn power-upów
                if get_time() - last_power_up > 10.0 && thread_rng().gen_range(0.0..1.0) < 0.005 {
                    let power_type = thread_rng().gen_range(0..2);
                    power_ups.push(PowerUp {
                        pos: vec2(thread_rng().gen_range(0.0..screen_width()), 0.0),
                                   speed: 80.0,
                                   alive: true,
                                   power_type,
                    });
                    last_power_up = get_time();
                }

                // Aktualizacja pocisków
                let mut bullets_to_kill = vec![];
                for (i, bullet) in bullets.iter_mut().enumerate() {
                    if bullet.alive {
                        bullet.pos.y += bullet.speed * get_frame_time();
                        if bullet.pos.y < 0.0 {
                            bullets_to_kill.push(i);
                        }
                    }
                }

                // Aktualizacja wrogów
                let mut enemies_to_kill = vec![];
                for (i, enemy) in enemies.iter_mut().enumerate() {
                    if enemy.alive {
                        enemy.pos.y += enemy.speed * get_frame_time();
                        if enemy.pos.y > screen_height() {
                            enemies_to_kill.push(i);
                        }
                    }
                }

                // Aktualizacja power-upów
                let mut power_ups_to_kill = vec![];
                for (i, power_up) in power_ups.iter_mut().enumerate() {
                    if power_up.alive {
                        power_up.pos.y += power_up.speed * get_frame_time();
                        if power_up.pos.y > screen_height() {
                            power_ups_to_kill.push(i);
                        }
                    }
                }

                // Kolizje pocisków z wrogami
                let mut collisions = vec![];
                for (b_idx, bullet) in bullets.iter().enumerate() {
                    if bullet.alive {
                        for (e_idx, enemy) in enemies.iter_mut().enumerate() {
                            if enemy.alive && (bullet.pos - enemy.pos).length() < 20.0 {
                                let damage = if bullet.bullet_type == 1 { 2 } else { 1 };
                                enemy.health -= damage;
                                if enemy.health <= 0 {
                                    collisions.push((b_idx, e_idx));
                                    score += 10 * (enemy.enemy_type as i32 + 1);
                                } else {
                                    bullets_to_kill.push(b_idx); // Tylko pocisk ginie jeśli wróg ma zdrowie
                                }
                            }
                        }
                    }
                }
                for (b_idx, e_idx) in collisions {
                    bullets[b_idx].alive = false;
                    enemies[e_idx].alive = false;
                }

                // Kolizje gracza z wrogami
                let mut player_hit = false;
                for (i, enemy) in enemies.iter().enumerate() {
                    if enemy.alive && (player_pos - enemy.pos).length() < 20.0 {
                        enemies_to_kill.push(i);
                        player_hit = true;
                    }
                }
                if player_hit {
                    player_health -= 1;
                    if player_health <= 0 {
                        if score > settings.high_score {
                            settings.high_score = score;
                            save_settings(&settings);
                        }
                        save_game(score, player_pos, settings.difficulty, &enemies, &bullets, &power_ups, player_health, level, double_shot);
                        game_state = GameState::Menu;
                    }
                }

                // Kolizje gracza z power-upami
                for (i, power_up) in power_ups.iter().enumerate() {
                    if power_up.alive && (player_pos - power_up.pos).length() < 20.0 {
                        power_ups_to_kill.push(i);
                        match power_up.power_type {
                            0 => player_health = (player_health + 1).min(5),
                            1 => double_shot = true,
                            _ => {},
                        }
                    }
                }

                // Postęp levelu
                if score > (level as i32 * 100) {
                    level += 1;
                }

                // Usuwanie martwych obiektów
                for &i in bullets_to_kill.iter().rev() {
                    bullets[i].alive = false;
                }
                for &i in enemies_to_kill.iter().rev() {
                    enemies[i].alive = false;
                }
                for &i in power_ups_to_kill.iter().rev() {
                    power_ups[i].alive = false;
                }
                bullets.retain(|b| b.alive);
                enemies.retain(|e| e.alive);
                power_ups.retain(|p| p.alive);

                // Rysowanie
                draw_rectangle(player_pos.x, player_pos.y, 20.0, 20.0, GREEN); // Gracz
                for bullet in bullets.iter() {
                    if bullet.alive {
                        draw_circle(bullet.pos.x, bullet.pos.y, 5.0, if bullet.bullet_type == 1 { ORANGE } else { YELLOW });
                    }
                }
                for enemy in enemies.iter() {
                    if enemy.alive {
                        let color = match enemy.enemy_type {
                            1 => BLUE, // Fast
                            2 => PURPLE, // Tank
                            _ => RED, // Normal
                        };
                        draw_rectangle(enemy.pos.x, enemy.pos.y, 20.0, 20.0, color);
                    }
                }
                for power_up in power_ups.iter() {
                    if power_up.alive {
                        let color = match power_up.power_type {
                            0 => GREEN, // Health
                            1 => GOLD, // Double shot
                            _ => WHITE,
                        };
                        draw_circle(power_up.pos.x, power_up.pos.y, 10.0, color);
                    }
                }

                draw_text(&format!("Score: {}", score), 10.0, 20.0, 20.0, WHITE);
                draw_text(&format!("Health: {}", player_health), 10.0, 40.0, 20.0, WHITE);
                draw_text(&format!("Level: {}", level), 10.0, 60.0, 20.0, WHITE);

                // Powrót do menu
                if is_key_pressed(KeyCode::Escape) {
                    if score > settings.high_score {
                        settings.high_score = score;
                        save_settings(&settings);
                    }
                    save_game(score, player_pos, settings.difficulty, &enemies, &bullets, &power_ups, player_health, level, double_shot);
                    game_state = GameState::Menu;
                }
            }
            GameState::Settings => {
                // Ustawienia
                draw_text("Settings", screen_width() / 2.0 - 50.0, 100.0, 40.0, WHITE);
                draw_text(
                    &format!("Difficulty: {:.1}", settings.difficulty),
                          screen_width() / 2.0 - 50.0,
                          200.0,
                          20.0,
                          WHITE,
                );
                if draw_button("+", screen_width() / 2.0 + 50.0, 200.0) {
                    settings.difficulty += 0.1;
                }
                if draw_button("-", screen_width() / 2.0 - 70.0, 200.0) {
                    settings.difficulty -= 0.1;
                    settings.difficulty = settings.difficulty.max(0.5);
                }
                draw_text(
                    &format!("Sound Volume: {:.1}", settings.sound_volume),
                          screen_width() / 2.0 - 50.0,
                          250.0,
                          20.0,
                          WHITE,
                );
                if draw_button("+", screen_width() / 2.0 + 50.0, 250.0) {
                    settings.sound_volume += 0.1;
                    settings.sound_volume = settings.sound_volume.min(1.0);
                }
                if draw_button("-", screen_width() / 2.0 - 70.0, 250.0) {
                    settings.sound_volume -= 0.1;
                    settings.sound_volume = settings.sound_volume.max(0.0);
                }
                if draw_button("Back", screen_width() / 2.0 - 50.0, 300.0) {
                    save_settings(&settings);
                    game_state = GameState::Menu;
                }
            }
            GameState::LoadGame => {
                // Wczytywanie gry
                draw_text("Load Game", screen_width() / 2.0 - 50.0, 100.0, 40.0, WHITE);
                if let Some(save) = load_game() {
                    draw_text(
                        &format!("Score: {}, Health: {}, Level: {}, Difficulty: {:.1}", save.score, save.player_health, save.level, save.difficulty),
                              screen_width() / 2.0 - 100.0,
                              200.0,
                              20.0,
                              WHITE,
                    );
                    if draw_button("Load", screen_width() / 2.0 - 50.0, 250.0) {
                        score = save.score;
                        player_pos = vec2(save.player_pos.x, save.player_pos.y);
                        settings.difficulty = save.difficulty;
                        player_health = save.player_health;
                        level = save.level;
                        double_shot = save.double_shot;
                        enemies = save.enemies.into_iter().map(|se| Enemy {
                            pos: vec2(se.pos.x, se.pos.y),
                                                               speed: se.speed,
                                                               alive: se.alive,
                                                               enemy_type: se.enemy_type,
                                                               health: if se.enemy_type == 2 { 3 } else { 1 },
                        }).collect();
                        bullets = save.bullets.into_iter().map(|sb| Bullet {
                            pos: vec2(sb.pos.x, sb.pos.y),
                                                               speed: sb.speed,
                                                               alive: sb.alive,
                                                               bullet_type: sb.bullet_type,
                        }).collect();
                        power_ups = save.power_ups.into_iter().map(|sp| PowerUp {
                            pos: vec2(sp.pos.x, sp.pos.y),
                                                                   speed: sp.speed,
                                                                   alive: sp.alive,
                                                                   power_type: sp.power_type,
                        }).collect();
                        game_state = GameState::Game;
                    }
                } else {
                    draw_text("No save found!", screen_width() / 2.0 - 50.0, 200.0, 20.0, WHITE);
                }
                if draw_button("Back", screen_width() / 2.0 - 50.0, 300.0) {
                    game_state = GameState::Menu;
                }
            }
        }

        next_frame().await;
    }
}

// Funkcja do rysowania przycisku
fn draw_button(text: &str, x: f32, y: f32) -> bool {
    let text_width = measure_text(text, None, 20, 1.0).width;
    let rect_width = text_width + 20.0;
    let rect_height = 30.0;
    let mouse_pos = mouse_position();
    let is_hovered = mouse_pos.0 > x && mouse_pos.0 < x + rect_width && mouse_pos.1 > y && mouse_pos.1 < y + rect_height;

    draw_rectangle(x, y, rect_width, rect_height, if is_hovered { GRAY } else { DARKGRAY });
    draw_text(text, x + 10.0, y + 20.0, 20.0, WHITE);

    is_hovered && is_mouse_button_pressed(MouseButton::Left)
}

// Funkcja zapisu gry
fn save_game(score: i32, player_pos: Vec2, difficulty: f32, enemies: &Vec<Enemy>, bullets: &Vec<Bullet>, power_ups: &Vec<PowerUp>, player_health: i32, level: u32, double_shot: bool) {
    let save = GameSave {
        score,
        player_pos: SerVec2 { x: player_pos.x, y: player_pos.y },
        difficulty,
        enemies: enemies.iter().map(|e| SerEnemy {
            pos: SerVec2 { x: e.pos.x, y: e.pos.y },
            speed: e.speed,
            alive: e.alive,
            enemy_type: e.enemy_type,
        }).collect(),
        bullets: bullets.iter().map(|b| SerBullet {
            pos: SerVec2 { x: b.pos.x, y: b.pos.y },
            speed: b.speed,
            alive: b.alive,
            bullet_type: b.bullet_type,
        }).collect(),
        power_ups: power_ups.iter().map(|p| SerPowerUp {
            pos: SerVec2 { x: p.pos.x, y: p.pos.y },
            speed: p.speed,
            alive: p.alive,
            power_type: p.power_type,
        }).collect(),
        player_health,
        level,
        double_shot,
    };
    let serialized = serde_json::to_string(&save).unwrap();
    fs::write("save.json", serialized).unwrap_or(());
}

// Funkcja wczytywania gry
fn load_game() -> Option<GameSave> {
    if let Ok(data) = fs::read_to_string("save.json") {
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

// Funkcja zapisu ustawień
fn save_settings(settings: &Settings) {
    let serialized = serde_json::to_string(settings).unwrap();
    fs::write("settings.json", serialized).unwrap_or(());
}

// Funkcja wczytywania ustawień
fn load_settings() -> Settings {
    if let Ok(data) = fs::read_to_string("settings.json") {
        serde_json::from_str(&data).unwrap_or(Settings {
            difficulty: 1.0,
            sound_volume: 0.5,
            high_score: 0,
        })
    } else {
        Settings {
            difficulty: 1.0,
            sound_volume: 0.5,
            high_score: 0,
        }
    }
}
