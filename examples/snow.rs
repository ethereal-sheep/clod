use std::f32::consts::PI;

use clod::{
    style::{CanvasAlignment, Stylize},
    App, AppResult,
};
use crossterm::style::Color;
use glam::{FloatExt, U16Vec2, Vec2};
use rand::Rng;

struct Entity {
    pos: Vec2,
    vel: Vec2,
    z: f32,
}

#[derive(Default)]
struct MyApp {
    rng: rand::rngs::ThreadRng,
    entities: Vec<Entity>,
    wind: Vec2,
    drop: Vec2,
    density: f32,
    wind_speed: f32,
    drop_speed: f32,
    accumulator: f32,
    is_paused: bool,
}

impl MyApp {
    fn create_droplet(&mut self, bounds: &U16Vec2) {
        self.entities.push(Entity {
            pos: Vec2 {
                x: self
                    .rng
                    .gen_range(-(bounds.y as f32)..(bounds.x + bounds.y) as f32),
                y: self.rng.gen_range(-2..0) as f32,
            },
            vel: self.drop.normalize() * self.drop_speed,
            z: self.rng.gen_range(0.0..=1.0),
        });
    }
}

fn ease_sin_sq(elapsed_ms: u128, duration_s: f32) -> f32 {
    let duration_ms = duration_s * 1000.0;
    let v = (((elapsed_ms as f32 % duration_ms) / duration_ms) * PI).sin();
    v * v
}
fn ease_cos_sq(elapsed_ms: u128, duration_s: f32) -> f32 {
    let duration_ms = duration_s * 1000.0;
    let v = (((elapsed_ms as f32 % duration_ms) / duration_ms) * PI).cos();
    v * v
}

impl App for MyApp {
    fn update(&mut self, state: &mut clod::State) -> Result<(), String> {
        let elapsed = state.elapsed_millis();
        self.drop = Vec2::new(-1.0, 1.0)
            .lerp(Vec2::new(1.0, 1.0), ease_cos_sq(elapsed, 40.0))
            .normalize();

        self.drop_speed = 0.60.lerp(1.0, ease_sin_sq(elapsed, 160.0));

        self.wind = Vec2::new(-1.0, 0.0)
            .lerp(Vec2::new(1.0, 0.0), ease_sin_sq(elapsed, 80.0))
            .normalize();

        self.density = 40.0.lerp(140.0, ease_sin_sq(elapsed, 160.0));

        let bounds = state.canvas.size();
        for entity in self.entities.iter_mut() {
            if !self.is_paused {
                entity.vel =
                    entity.vel + self.wind * state.delta_seconds() as f32 * self.wind_speed;
                entity.pos = entity.pos
                    + entity.vel * state.delta_seconds() as f32 * (20.0 + (60.0 * entity.z));
            }
            let gray_value = (entity.z * 100.0) as u8 + 40;
            if (entity.pos.y as u16) < bounds.y {
                if entity.pos.x >= 0.0 {
                    state.canvas.draw_with_color(
                        entity.pos.as_u16vec2(),
                        Color::Rgb {
                            r: gray_value,
                            g: gray_value,
                            b: gray_value,
                        },
                    );
                }
            }
        }

        self.entities.retain(|e| {
            (e.pos.y as u16) < bounds.y + bounds.y
                && (e.pos.x as u16) < bounds.x + bounds.y
                && e.pos.y > -(bounds.y as f32)
                && e.pos.x > -(bounds.y as f32)
        });

        if !self.is_paused {
            self.accumulator += state.delta_seconds() as f32;
            let drops = self.density * self.accumulator;
            self.accumulator = drops.fract() / self.density;
            for _ in 0..drops as usize {
                self.create_droplet(&bounds);
            }
        } else {
            state.canvas.print(
                "Paused"
                    .bold()
                    .italic()
                    .align(CanvasAlignment::CENTER)
                    .border_white()
                    .vertical_padding(1)
                    .on_red()
                    .horizontal_padding(3),
            );
        }

        Ok(())
    }

    fn on_key_event(&mut self, _state: &mut clod::State, event: crossterm::event::KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Up => {
                self.wind = (self.wind + Vec2::new(0.0, -0.1)) * 0.9;
            }
            crossterm::event::KeyCode::Down => {
                self.wind = (self.wind + Vec2::new(0.0, 0.1)) * 0.9;
            }
            crossterm::event::KeyCode::Left => {
                self.wind = (self.wind + Vec2::new(-0.1, 0.0)) * 0.9;
            }
            crossterm::event::KeyCode::Right => {
                self.wind = (self.wind + Vec2::new(0.1, 0.0)) * 0.9;
            }
            crossterm::event::KeyCode::Char('r') => {
                self.wind = Vec2::ZERO;
            }
            crossterm::event::KeyCode::Char('p') => {
                self.is_paused = !self.is_paused;
            }
            _ => (),
        }
    }

    fn init(&mut self, _state: &mut clod::State) -> Result<(), String> {
        self.drop_speed = 1.0;
        self.wind_speed = 0.2;
        self.density = 100.0;
        self.drop = Vec2 { x: 1.0, y: 1.0 }.normalize();
        Ok(())
    }
}

fn main() -> AppResult {
    let mut app = MyApp::default();
    app.run()
}

#[cfg(test)]
mod tests {}
