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

use ggez::*;
use ggez::graphics::Color;
use ggez::input::keyboard::KeyInput;
use winit::keyboard::{Key, NamedKey};
use std::f32::consts::PI;
use ggez::conf::FullscreenType;

const RAY_MAX: f32 = 512.0;

const PROGRESSION_SCORE: f32 = 1.0;
const LAP_SCORE_SPEED: f32 = 1.0;
const SLIDE_PENALTY: f32 = 1.0;
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


    let track = Track::from_file("tracks/3.txt");
    track.render("target/debug/resources/track.png");
    
    let mut bots = Vec::new();
    for _i in 0..128 {
        bots.push(
            AICar {
                car: Car {
                    position: track.start_finish.get_point(track.start_position),
                    rotation: track.start_finish.get_bearing(track.start_position),
                    velocity: 0.0,
                    stats: CarStats {
                        base_tyre_grip: 500.0,
                        power: 250.0,
                        drag_coeff: 1.0 / 1000.0,
                        turn_radius: 50.0,
                        downforce_coeff: 1.0 / 3000.0,
                        hitpoints: vec![Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 11.0, y: 0.0 }, Vec2 { x: -11.0, y: 0.0 }, Vec2 { x: 11.0, y: 40.0 }, Vec2 { x: -11.0, y: 40.0 }],
                    },
                    image: graphics::Image::from_path(&mut ctx, "/car.png").unwrap(),
                },
                network: Network::new(vec![10, 6, 6, 2], vec![Function::Relu, Function::Relu, Function::Tanh]),
                score: 0.0,
                alive: false,
                distance_traveled: 0.0,
                progress: 0.0,
                laps: 0,
            }
        );
    }
    let state = State {
        track: GameTrack {
            track,
            image: graphics::Image::from_path(&mut ctx, "/track.png").unwrap(),
        },
        bots,
        time_since_start: 0.0,
        is_fullscreen: false,
    }; 

    let _ = event::run(ctx, event_loop, state); 
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(60) {
            let dt = 1.0 / 60.0;
            self.time_since_start += dt;
            for bot in &mut self.bots {
                if !bot.alive {
                    continue;
                }
                bot.update(&self.track, self.time_since_start, dt);
            }

            if self.bots.iter().all(|bot| !bot.alive || (bot.car.velocity < 25.0 && self.time_since_start > 1.0) || bot.laps > 3) {
                self.reset();
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
        canvas.finish(ctx)
    }
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, repeated: bool) -> GameResult {
        match input.event.logical_key {
            Key::Named(NamedKey::Space) if !repeated => self.reset(),
        Key::Named(NamedKey::F11) if !repeated => {
            self.is_fullscreen = !self.is_fullscreen;
            
            let fullscreen_type = if self.is_fullscreen {
                FullscreenType::Desktop
            } else {
                FullscreenType::Windowed
            };
            
            ctx.gfx.set_fullscreen(fullscreen_type)?;
        }
        
        _ => {}
    }
    Ok(())
    }
}

impl State {
    fn reset(&mut self) {
        self.bots.sort_by(|a, b| b.score.total_cmp(&a.score));

        for i in (16..self.bots.len()).rev() {
            self.bots[i].network = self.bots[i % 16].network.vary();
        }

        let position = self.track.track.start_finish.get_point(self.track.track.start_position);
        let bearing = self.track.track.start_finish.get_bearing(self.track.track.start_position);
        for bot in &mut self.bots {
            bot.car.position = position;
            bot.car.rotation = bearing;
            bot.car.velocity = 0.0;
            bot.score = 0.0;
            bot.alive = true;
            bot.distance_traveled = 0.0;
            bot.progress = 0.0;
            bot.laps = 0;
        }
        self.time_since_start = 0.0;
    }
}

struct State {
    track: GameTrack,
    bots: Vec<AICar>,
    time_since_start: f32,
    is_fullscreen: bool,
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
}
impl AICar {
    fn update(&mut self, track: &GameTrack, time_since_start: f32, dt: f32) {
        let mut inputs= Vec::new();

        inputs.push(self.car.velocity / 500.0);

        for i in 0..=8 {
            let angle = self.car.rotation - PI / 2.0 + PI * i as f32 / 8.0;
            inputs.push(track.ray(self.car.position, angle));
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

        let last_progress = self.progress;
        self.progress = track.track.get_progression(self.car.position);
        if last_progress > 0.875 && self.progress < 0.125 {
            self.laps += 1;
            self.score += (self.distance_traveled / time_since_start) * LAP_SCORE_SPEED;
            self.score += 1.0;
        }
        self.score += (self.progress - last_progress) * PROGRESSION_SCORE;
    }
    fn draw(&self, canvas: &mut graphics::Canvas) {
        self.car.draw(canvas);
    }
}