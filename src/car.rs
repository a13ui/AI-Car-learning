use crate::vectors::*;
use ggez::*;

pub struct Car {
    pub transform: Transform,
    pub velocity: f32,

    pub stats: CarStats,

    pub image: graphics::Image,
}
pub struct CarStats {
    pub base_tyre_grip: f32,
    pub power: f32,
    pub drag_coeff: f32,
    pub turn_radius: f32,
    pub downforce_coeff: f32,
    pub hitpoints: Vec<Vec2>,
    
    pub top_speed: f32,
}

impl Car {
    pub fn update(&mut self, controls: CarControl, dt: f32) -> bool {
        let velocity_squared = self.velocity * self.velocity;
        let grip = self.stats.base_tyre_grip + self.stats.downforce_coeff * velocity_squared;

        let travel_dir = if self.velocity > 0.0 {
            1.0
        } else if self.velocity < 0.0 {
            -1.0
        } else {
            0.0
        };

        let mut rotation_delta = 0.0;

        if self.velocity != 0.0 {
            let radius = (velocity_squared / grip).max(self.stats.turn_radius);
            let rad_sec = self.velocity / radius;
            rotation_delta += controls.turn * rad_sec;
        }

        let lateral_delta = rotation_delta.abs() * self.velocity.abs();

        let mut velocity_delta = 0.0;

        velocity_delta -= travel_dir * controls.brake * grip;
        velocity_delta -= travel_dir * velocity_squared * self.stats.drag_coeff;
        velocity_delta += controls.accelerate * self.stats.power;

        
        if (lateral_delta * lateral_delta + velocity_delta * velocity_delta) > grip * grip {
            self.transform.position += Vec2::from_bearing(self.transform.bearing) * self.velocity * dt;
            false
        }
        else {
            self.velocity += velocity_delta * dt;
            self.velocity = self.velocity.max(0.0);
            self.transform.bearing += rotation_delta * dt;
            self.transform.position += Vec2::from_bearing(self.transform.bearing) * self.velocity * dt;
            true
        }

    }
    pub fn draw(&self, canvas: &mut graphics::Canvas) {
        canvas.draw(&self.image, graphics::DrawParam::default()
            .dest([self.transform.position.x, 1080.0 - self.transform.position.y])
            .offset([0.5, 0.5])
            .scale([1.0, 1.0])
            .rotation(self.transform.bearing),
        );
    }
}

impl CarStats {
    pub fn f2004() -> Self {
        CarStats {
            base_tyre_grip: 1000.0,
            power: 250.0,
            drag_coeff: 1.0 / 2000.0,
            turn_radius: 50.0,
            downforce_coeff: 1.0 / 2000.0,
            hitpoints: vec![Vec2 { x: 0.0, y: 0.0 }, Vec2 { x: 11.0, y: 0.0 }, Vec2 { x: -11.0, y: 0.0 }, Vec2 { x: 11.0, y: 40.0 }, Vec2 { x: -11.0, y: 40.0 }],
            top_speed: (250.0 * 2000.0 as f32).sqrt(),
        }
    }
}
pub struct CarControl {
    pub turn: f32,
    pub accelerate: f32,
    pub brake: f32,
}