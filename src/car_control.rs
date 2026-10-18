use ggez::winit::keyboard::{Key, SmolStr};
use input::keyboard::KeyboardContext;
use ggez::*;

use crate::vectors::Vec2;

pub struct CarControl {
    pub turn: f32,
    pub accelerate: f32,
    pub brake: f32,
}

impl CarControl {
    fn from_vec2(vector: Vec2) -> Self {
        CarControl {
            turn: vector.x,
            accelerate: vector.y.max(0.0),
            brake: vector.y.min(0.0).abs(),
        }
    }
}

pub fn keyboard_control(keyboard: &mut KeyboardContext) -> CarControl {
    CarControl::from_vec2(Vec2 {
        x: {
            let mut turn = 0.0;
            if is_key_down(keyboard, "d") {
                turn += 1.0;
            }
            if is_key_down(keyboard, "a") {
                turn -= 1.0;
            }
            turn
        },
        y: {
            let mut forward = 0.0;
            if is_key_down(keyboard, "w") {
                forward += 1.0;
            }
            if is_key_down(keyboard, "s") {
                forward -= 1.0;
            }
            forward
        }
    }.normalize())
}

pub fn is_key_down(keyboard: &mut KeyboardContext, key: &str) -> bool {
    keyboard.is_logical_key_pressed(&Key::Character(SmolStr::new_inline(key)))
}