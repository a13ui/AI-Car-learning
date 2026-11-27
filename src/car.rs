use crate::vectors::*;
use ggez::*;
const GRAVITY: f32 = 9.80665;

struct Car {
    transform: Transform,
    velocity: f32,

    hitpoints: Vec<Vec2>, 

    engine: Engine,
    gears: GearBox,
    tyres: Tyres,
    aero: Aero,
    mass: f32,
    turn_radius: f32,
}

impl Car {
    fn update(&mut self, controls: CarControl, dt: f32) -> bool {
        if controls.downshift { self.try_downshift(); } 
        if controls.upshift { self.try_upshift(); }

        let velocity_sq = self.velocity * self.velocity;

        let grip = self.tyres.friction_coeff * (self.mass * GRAVITY + self.aero.downforce_coeff * velocity_sq);

        let travel_dir = if self.velocity == 0.0 { 0.0 } else { self.velocity.signum() };

        let mut longitudinal_force_delta = 0.0;

        longitudinal_force_delta -= travel_dir * controls.brake * grip;
        longitudinal_force_delta -= travel_dir * self.aero.drag_coeff * velocity_sq;
        longitudinal_force_delta += self.engine.update(controls.throttle, dt);

        let mut lateral_force_delta = 0.0;

        let mut turn_radius = 0.0;
        if controls.turn != 0.0 {
            turn_radius = self.turn_radius / controls.turn;
            lateral_force_delta = velocity_sq / turn_radius;
        }

        let used_grip_sq = lateral_force_delta * lateral_force_delta + longitudinal_force_delta * longitudinal_force_delta;
        let has_slid = used_grip_sq > grip * grip;
        if has_slid {
            let used_grip = used_grip_sq.sqrt();
            let multiplier = grip / used_grip;

            longitudinal_force_delta *= multiplier;
            lateral_force_delta *= multiplier;

            turn_radius = velocity_sq / lateral_force_delta;
        }

        self.velocity += (longitudinal_force_delta / self.mass) * dt;
        self.velocity = self.velocity.max(0.0);
        if turn_radius != 0.0 {
            self.transform.bearing += self.velocity / turn_radius * dt;
        }
        self.limit_rpm();
        self.update_position(dt);
        has_slid
    }
    fn try_upshift(&mut self) -> bool {
        self.try_shift(1)
    }
    fn try_downshift(&mut self) -> bool {
        self.try_shift(-1)
    }
    fn try_shift(&mut self, change: i32) -> bool {
        let new_gear = self.gears.gear + change as usize;
        let new_rpm = self.velocity / self.gears.get_ratio(new_gear);

        if self.engine.validate_rpm(new_rpm) || new_gear == 1 {
            self.gears.shift(change)
        }
        else { false }
    }
    fn limit_rpm(&mut self) {
        let gear_ratio = self.gears.get_current_ratio();
        self.engine.rpm = self.velocity / gear_ratio;
        if self.engine.rpm > self.engine.rpm_limit {
            self.velocity = gear_ratio * self.engine.rpm_limit;
            self.engine.rpm = self.engine.rpm_limit;
        }
    }
    fn update_position(&mut self, dt: f32) {
        self.transform.position += Vec2::from_bearing(self.transform.bearing) * self.velocity * dt;

    }
}

struct Engine {
    base_power: f32,
    rpm: f32,
    rpm_limit: f32,
    idle_rpm: f32,

    turbos: Vec<Turbo>,

    super_pressure: f32,
}

impl Engine {
    fn update(&mut self, throttle: f32, dt: f32) -> f32 {

        let intake_pressure = self.get_intake_pressure();

        let airflow = throttle * self.rpm.clamp(self.idle_rpm, self.rpm_limit) * intake_pressure;
        for turbo in &mut self.turbos {
            turbo.spool(airflow, dt);
        }
        
        airflow * self.base_power
    }

    fn get_intake_pressure(&mut self) -> f32 {
        let mut intake_pressure = 1.0 + self.super_pressure;
        for turbo in &self.turbos {
            intake_pressure += turbo.boost();
        }
        intake_pressure
    }
    fn validate_rpm(&self, rpm: f32) -> bool {
        self.idle_rpm < rpm && rpm < self.rpm_limit
    }
}
struct Turbo  {
    spool: f32,
    boost: f32,
    spool_rate: f32,
}
impl Turbo {
    fn spool(&mut self, airflow: f32, dt: f32) {
        self.spool += (airflow - self.spool) * self.spool_rate * dt;
    }
    fn boost(&self) -> f32 {
        self.spool * self.boost
    }
}

struct GearBox {
    ratios: Vec<f32>,
    gear: usize,
}

impl GearBox {
    fn get_ratio(&self, gear: usize) -> f32 {
        if gear < 0 {
            self.ratios[0]
        }
        else if gear < self.ratios.len() {
            self.ratios[gear]
        }
        else {
            self.ratios[self.ratios.len() - 1]
        }
    }
    fn get_current_ratio(&self) -> f32 {
        self.ratios[self.gear]
    }
    fn shift(&mut self, change: i32) -> bool {
        let new_gear = self.gear as i32 + change;
        if 0 <= new_gear && new_gear < self.ratios.len()  as i32 {
            self.gear = (self.gear as i32 + change) as usize;
            true
        }
        else { false }
    }
}

struct Aero {
    drag_coeff: f32,
    downforce_coeff: f32,
}
struct Tyres {
    friction_coeff: f32,
}

struct CarControl {
    turn: f32,
    throttle: f32,
    brake: f32,
    upshift: bool,
    downshift: bool,
}