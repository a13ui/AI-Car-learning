mod neural;
mod vectors;
mod car_control;
mod car;
mod track;

use car::{Car, CarStats};

use track::Track;

use neural::{Network, Function};

use car_control::*;

use vectors::Vec2;

use ggez::{input::keyboard, *};
use ggez::graphics::Color;
use ggez::input::keyboard::KeyInput;
use winit::keyboard::{Key, NamedKey, SmolStr};
use std::f32::consts::PI;
use ggez::conf::FullscreenType;
use input::keyboard::KeyboardContext;

const RAY_MAX: f32 = 512.0;

const PROGRESSION_SCORE: f32 = 100.0; // score made per lap of progression
const LAP_SCORE_SPEED: f32 = 500.0; // score made per lap divided by lap time in seconds
const SLIDE_PENALTY: f32 = 1.0; // penalty for traction limit per second
fn main() {
    let c = conf::Conf {
        window_mode: conf::WindowMode {
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let (mut ctx, event_loop) = ContextBuilder::new("game", "")
        .default_conf(c)
        .build()
        .unwrap();

    let track = Track::from_file("tracks/1.txt");
    track.render("target/debug/resources/track.png");
    
    let mut bots = Vec::new();
    for _i in 0..128 {
        bots.push(
            AICar {
                car: Car {
                    position: track.start_finish.get_point(track.start_position),
                    rotation: track.start_finish.get_bearing(track.start_position),
                    velocity: 0.0,
                    stats: CarStats::f2004(),
                    image: graphics::Image::from_path(&mut ctx, "/car.png").unwrap(),
                },
                network: Network::new(vec![10, 5, 5, 2], vec![Function::Relu, Function::Relu, Function::Tanh]),
                score: 0.0,
                alive: false,
                distance_traveled: 0.0,
                progress: 0.0,
                laps: 0,
                time_since_last_lap: 0.0,
            }
        );
    }
    let state = State {
        track: GameTrack {
            track,
            image: graphics::Image::from_path(&mut ctx, "/track.png").unwrap(),
        },
        bots,
        player: PlayerCar {
            car: Car {
                position: Vec2 { x: 0.0, y: 0.0 },
                rotation: 0.0,
                velocity: 0.0,
                stats: CarStats::f2004(),
                image: graphics::Image::from_path(&mut ctx, "/car.png").unwrap(),
            },
            controls: Vec2 { x: 0.0, y: 0.0 }
        },
        time_since_start: 0.0,
        is_fullscreen: false,
        fps: 60.0,
    }; 

    let _ = event::run(ctx, event_loop, state); 
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(self.fps as u32) {
            let dt = 1.0 / 60.0;
            self.player.update(&mut ctx.keyboard, dt);
            self.time_since_start += dt;
            for bot in &mut self.bots {
                if !bot.alive {
                    continue;
                }
                bot.update(&self.track, dt);
            }

            if self.bots.iter().all(|bot| !bot.alive || (bot.car.velocity < 25.0 && self.time_since_start > 1.0) || bot.laps >= 3) {
                self.next_generation();
            }
        }

        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        self.track.draw(&mut canvas);

        for bot in &self.bots {
            bot.draw(&mut canvas);
        }

        self.player.draw(&mut canvas);        
        canvas.finish(ctx)
    }
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeated: bool) -> GameResult {
        match input.event.logical_key {
            Key::Named(NamedKey::Space) if !repeated => self.next_generation(),
            Key::Named(NamedKey::F11) if !repeated => {
                self.is_fullscreen = !self.is_fullscreen;
                
                let fullscreen_type = if self.is_fullscreen {
                    FullscreenType::Desktop
                } else {
                    FullscreenType::Windowed
                };
                
                ctx.gfx.set_fullscreen(fullscreen_type)?;
            }
            Key::Named(NamedKey::ArrowLeft) if !repeated => {
                self.fps = 60.0;
            },
            Key::Named(NamedKey::ArrowDown) if !repeated => {
                if self.fps > 60.0 { self.fps -= 60.0; };
            }
            Key::Named(NamedKey::ArrowUp) if !repeated => {
                self.fps += 60.0;
            }
        _ => {}
    }
    Ok(())
    }
}

impl State {
    fn next_generation(&mut self) {
        self.bots.sort_by(|a, b| b.score.total_cmp(&a.score));

        for i in (16..self.bots.len()).rev() {
            self.bots[i].network = self.bots[i % 16].network.vary();
        }

        let position = self.track.track.start_finish.get_point(self.track.track.start_position);
        let bearing = self.track.track.start_finish.get_bearing(self.track.track.start_position);

        self.player.car.position = position;
        self.player.car.rotation = bearing;
        self.player.car.velocity = 0.0;
        for bot in &mut self.bots {
            bot.car.position = position;
            bot.car.rotation = bearing;
            bot.car.velocity = 0.0;
            bot.score = 0.0;
            bot.alive = true;
            bot.distance_traveled = 0.0;
            bot.progress = 0.0;
            bot.laps = 0;
            bot.time_since_last_lap = 0.0;
        }

        self.time_since_start = 0.0;
    }
}

struct State {
    track: GameTrack,
    bots: Vec<AICar>,
    player: PlayerCar,
    is_fullscreen: bool,
    time_since_start: f32,
    fps: f32,
}
struct GameTrack {
    track: Track,
    image: graphics::Image,
}
impl GameTrack {
    fn draw(&self, canvas: &mut graphics::Canvas) {
        canvas.draw(&self.image, graphics::DrawParam::default());
    }
    fn collide(&self, car: &Car) -> bool {
        car.stats.hitpoints.iter().all(|point| self.track.contains(car.position + point.rotate(car.rotation)))
    }
    fn ray(&self, from: Vec2, bearing: f32) -> f32 {
        self.track
            .collide_ray(from, Vec2::from_bearing(bearing))
            .map_or(0.0, |distance| distance.min(RAY_MAX) / RAY_MAX)
    }
}

struct AICar {
    car: Car,
    network: Network,
    score: f32,
    alive: bool,
    distance_traveled: f32,
    progress: f32,
    laps: i32,

    time_since_last_lap: f32,
}

impl AICar {
    fn update(&mut self, track: &GameTrack, dt: f32) {
        let mut inputs= Vec::new();

        inputs.push(self.car.velocity / self.car.stats.top_speed);

        let ray_start = self.car.position + Vec2 {x: 0.0, y: 20.0};
        for i in 0..=8 {
            let angle = self.car.rotation - PI / 2.0 + PI * i as f32 / 8.0;
            inputs.push(track.ray(ray_start, angle));
        }

        let out = self.network.run(&inputs);
        let controls = CarControl { turn: out[0], accelerate: out[1].max(0.0), brake: (-out[1]).max(0.0)};

        if !self.car.update(controls, dt) {
            self.score -= SLIDE_PENALTY * dt;
        }
        self.distance_traveled += self.car.velocity * dt;

        if !track.collide(&self.car) {
            self.car.velocity = 0.0;
            self.alive = false;
            return;
        }

        self.time_since_last_lap += dt;

        let last_progress = self.progress;
        self.progress = track.track.get_progression(self.car.position);

        let delta = (self.progress - last_progress + 0.5).rem_euclid(1.0) - 0.5;
        self.score += delta * PROGRESSION_SCORE;

        if last_progress > 0.875 && self.progress < 0.125 {
            self.laps += 1;
            self.score += LAP_SCORE_SPEED / self.time_since_last_lap;
            self.time_since_last_lap = 0.0;
        }

    }
    fn draw(&self, canvas: &mut graphics::Canvas) {
        self.car.draw(canvas);
    }
}

struct PlayerCar {
    car: Car,
    controls: Vec2,
}
impl PlayerCar {
    fn update(&mut self, keyboard: &mut keyboard::KeyboardContext, dt: f32) {
        let sens = 1.0 / 10.0;
        if is_key_down(keyboard, "a") {
            self.controls.x -= (-1.0 - self.controls.x).abs() * sens
        }
        else if is_key_down(keyboard, "d") {
            self.controls.x += (1.0 - self.controls.x).abs() * sens
        }
        else {
            self.controls.x += (0.0 - self.controls.x).abs() * sens
        }
        if is_key_down(keyboard, "s") {
            self.controls.y -= (-1.0 - self.controls.y).abs() * sens
        }
        else if is_key_down(keyboard, "w") {
            self.controls.y += (1.0 - self.controls.y).abs() * sens
        }
        else {
            self.controls.y += (0.0 - self.controls.y).abs() * sens
        }
        self.car.update(CarControl::from_vec2(self.controls), dt);
    }
    fn draw(&self, canvas: &mut graphics::Canvas) {
        self.car.draw(canvas);
    }
}

pub fn is_key_down(keyboard: &mut KeyboardContext, key: &str) -> bool {
    keyboard.is_logical_key_pressed(&Key::Character(SmolStr::new_inline(key)))
}