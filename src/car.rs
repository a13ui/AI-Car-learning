use crate::vectors::*;
use ggez::*;

const GRAVITY: f32 = 9.80665;
pub struct Car {
    pub transform: Transform,
    pub velocity: f32,
    
    pub engine_revs: f32,
    pub turbo_spool: f32,
    pub gear: usize,
    pub stats: CarStats,

    pub image: graphics::Image,
}
pub struct CarStats {
    pub tyre_friction_coeff: f32,
    pub base_engine_power: f32,

    pub rev_limit: f32,
    pub turbo_boost: f32,
    pub turbo_spool_rate: f32,
    pub super_pressure: f32,
    
    pub gear_ratios: Vec<f32>,

    pub mass: f32,
    pub drag_coeff: f32,
    pub turn_radius: f32,
    pub downforce_coeff: f32,
    pub hitpoints: Vec<Vec2>, 

    pub top_speed: f32,
}

impl Car {
    pub fn update(&mut self, controls: CarControl, dt: f32) -> bool {
        let velocity_sq = self.velocity * self.velocity;

        let normal_reaction = self.stats.mass * GRAVITY + self.stats.downforce_coeff * velocity_sq;
        let grip = self.stats.tyre_friction_coeff * normal_reaction;

        if controls.downshift {
            if self.gear > 0 && self.velocity / self.stats.gear_ratios[self.gear - 1] < self.stats.rev_limit {
                self.gear -= 1;
            }
        } 
        if controls.upshift {
            if self.gear < self.stats.gear_ratios.len() - 1 {
                self.gear += 1;
            }
        }

        let power_output;
        if self.gear > 0 {
            let intake_pressure = 1.0 + self.stats.super_pressure + self.turbo_spool * self.stats.turbo_boost;
            let airflow = controls.accelerate * self.engine_revs.max(1000.0) * intake_pressure;
            power_output = airflow * self.stats.base_engine_power;
            self.turbo_spool += (airflow - self.turbo_spool) * self.stats.turbo_spool_rate * dt;
        }
        else { power_output = 0.0; }

        let mut velocity_force_delta = 0.0;

        let travel_dir = if self.velocity == 0.0 { 0.0 } else  { self.velocity.signum() };

        velocity_force_delta -= travel_dir * controls.brake * grip;
        velocity_force_delta -= travel_dir * velocity_sq * self.stats.drag_coeff;
        velocity_force_delta += power_output; 

        let mut lateral_force_delta = 0.0;

        let mut turn_radius = 0.0;
        if controls.turn != 0.0 {
            turn_radius = self.stats.turn_radius / controls.turn;
            lateral_force_delta = velocity_sq / turn_radius;
        }

        let used_grip_sq = lateral_force_delta * lateral_force_delta + velocity_force_delta * velocity_force_delta;
        let has_slid = used_grip_sq > grip * grip;
        if has_slid {
            let used_grip = used_grip_sq.sqrt();
            let multiplier = grip / used_grip;

            velocity_force_delta *= multiplier;
            lateral_force_delta *= multiplier;

            turn_radius = velocity_sq / lateral_force_delta;
        }

        let rotation_delta = if turn_radius != 0.0 { self.velocity / turn_radius } else { 0.0 };

        self.velocity += (velocity_force_delta  / self.stats.mass) * dt;
        self.velocity = self.velocity.max(0.0);
        self.transform.bearing += rotation_delta * dt;

        self.transform.position += Vec2::from_bearing(self.transform.bearing) * self.velocity * dt;

        let gear_ratio = self.stats.gear_ratios[self.gear];

        self.engine_revs = self.velocity / gear_ratio;
        if self.engine_revs > self.stats.rev_limit {
            self.velocity = gear_ratio * self.stats.rev_limit;
            self.engine_revs = self.stats.rev_limit;
        }
        has_slid
    }
    pub fn draw(&self, canvas: &mut graphics::Canvas) {
        canvas.draw(&self.image, graphics::DrawParam::default()
            .dest([self.transform.position.x, 1080.0 - self.transform.position.y])
            .offset([0.5, 0.5])
            .scale([1.0, 1.0])
            .rotation(self.transform.bearing),
        );
    }
    pub fn f2004(ctx: &mut Context, position: Transform) -> Self {
        Car {
            transform: position,
            velocity: 0.0,
            engine_revs: 0.0,
            turbo_spool: 0.0,
            gear: 0,
            stats: CarStats::f2004(),
            image: graphics::Image::from_path(ctx, "/car.png").unwrap(),
        }
    }
}

impl CarStats {
    pub fn f2004() -> Self {
        CarStats {
            tyre_friction_coeff: 1000.0,
            base_engine_power: 250.0,
            rev_limit: 20000.0,
            turbo_boost: 0.0,
            turbo_spool_rate: 0.0,
            super_pressure: 0.0,
            gear_ratios: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
            mass: 600.0,
            drag_coeff: 1.0 / 2000.0,
            turn_radius: 50.0,
            downforce_coeff: 1.0 / 250.0,
            hitpoints: vec![Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 11.0, y: 0.0 }, Vec2 { x: -11.0, y: 0.0 }, Vec2 { x: 11.0, y: 40.0 }, Vec2 { x: -11.0, y: 40.0 }],
            top_speed: (250.0 * 2000.0 as f32).sqrt(),
        }
    }
}
pub struct CarControl {
    pub turn: f32,
    pub accelerate: f32,
    pub brake: f32,
    pub upshift: bool,
    pub downshift: bool,
}