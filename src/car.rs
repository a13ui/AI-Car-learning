use crate::vectors::Vec2;
use crate::car_control::CarControl;
use ggez::*;

pub struct Car {
    pub position: Vec2,
    pub rotation: f32,
    pub velocity: f32,

    pub base_tyre_grip: f32,
    pub power: f32,
    pub drag_coeff: f32,
    pub turn_radius: f32,
    pub downforce_coeff: f32,

    pub image: graphics::Image,
}

impl Car {
    pub fn update(&mut self, controls: CarControl, dt: f32) {
        let velocity_squared = self.velocity * self.velocity;
        let grip = self.base_tyre_grip + self.downforce_coeff * velocity_squared;

        let travel_dir = if self.velocity > 0.0 {
            1.0
        } else if self.velocity < 0.0 {
            -1.0
        } else {
            0.0
        };

        let mut rotation_delta = 0.0;

        if self.velocity != 0.0 {
            let radius = (velocity_squared / grip).max(self.turn_radius);
            let rad_sec = self.velocity / radius;
            rotation_delta += controls.turn * rad_sec;
        }

        let mut velocity_delta = 0.0;

        velocity_delta -= travel_dir * controls.brake * grip;
        velocity_delta -= travel_dir * velocity_squared * self.drag_coeff;
        velocity_delta += controls.accelerate * self.power;

        let lateral_delta = rotation_delta.abs() * self.velocity.abs();
        
        if (lateral_delta * lateral_delta + velocity_delta * velocity_delta) < grip * grip {
            self.velocity += velocity_delta * dt;
            self.velocity = self.velocity.max(0.0);
            self.rotation += rotation_delta * dt;
        }
        self.position += Vec2::from_bearing(self.rotation) * self.velocity * dt;
    }
    pub fn draw(&self, canvas: &mut graphics::Canvas) {
        canvas.draw(&self.image, graphics::DrawParam::default()
            .dest([self.position.x, 1080.0 - self.position.y])
            .offset([0.5, 0.5])
            .scale([1.0, 1.0])
            .rotation(self.rotation),
        );
    }
}