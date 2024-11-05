use std::collections::VecDeque;

use clod::{App, AppResult};
use crossterm::style::Color;
use glam::{U16Vec2, Vec2};
use rand::Rng;

#[derive(Default)]
struct Snake {
    parts: VecDeque<U16Vec2>,
    position: Vec2,
    velocity: Vec2,
}

impl Snake {
    fn init(&mut self, head: U16Vec2) {
        for i in (0..20).rev() {
            self.parts.push_front(U16Vec2::new(head.x - i, head.y));
        }
        self.position = self.parts.front().unwrap().as_vec2();
        self.velocity = Vec2::new(1.0, 0.0);
    }
}

#[derive(Default)]
struct MyApp {
    rng: rand::rngs::ThreadRng,
    snake: Snake,
    apple: U16Vec2,
    is_paused: bool,
}

impl MyApp {
    fn random_position(&mut self, bounds: &U16Vec2) -> U16Vec2 {
        U16Vec2::new(
            self.rng.gen_range(0..bounds.x),
            self.rng.gen_range(0..bounds.y),
        )
    }
}

impl App for MyApp {
    fn init(&mut self, state: &mut clod::State) -> Result<(), String> {
        let bounds = state.canvas.size();
        let center = bounds / 2;
        self.snake.init(center);
        self.apple = self.random_position(&bounds);
        Ok(())
    }

    fn update(&mut self, state: &mut clod::State) -> Result<(), String> {
        let bounds = state.canvas.size();
        if !self.is_paused {
            let new_head = self.snake.position
                + bounds.as_vec2()
                + self.snake.velocity * state.delta_seconds() as f32 * 30.0;
            self.snake.position =
                Vec2::new(new_head.x % bounds.x as f32, new_head.y % bounds.y as f32);
            if self.snake.parts.front() != Some(&self.snake.position.as_u16vec2()) {
                let new_head = self.snake.position.as_u16vec2();
                self.snake.parts.push_front(new_head);

                if self.apple == new_head {
                    self.apple = self.random_position(&bounds);
                } else {
                    self.snake.parts.pop_back();
                }
            }
        }

        for (i, part) in self.snake.parts.iter_mut().enumerate() {
            if i % 2 == 0 {
                state.canvas.draw_with_color(*part, Color::Cyan);
            } else {
                state.canvas.draw_with_color(*part, Color::Blue);
            }
        }
        state.canvas.draw_with_color(self.apple, Color::Red);

        Ok(())
    }

    fn on_key_event(&mut self, _state: &mut clod::State, event: crossterm::event::KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Up => {
                if self.snake.velocity.y == 0.0 && !self.is_paused {
                    self.snake.velocity = Vec2::new(0.0, -1.0);
                }
            }
            crossterm::event::KeyCode::Down => {
                if self.snake.velocity.y == 0.0 && !self.is_paused {
                    self.snake.velocity = Vec2::new(0.0, 1.0);
                }
            }
            crossterm::event::KeyCode::Left => {
                if self.snake.velocity.x == 0.0 && !self.is_paused {
                    self.snake.velocity = Vec2::new(-1.0, 0.0);
                }
            }
            crossterm::event::KeyCode::Right => {
                if self.snake.velocity.x == 0.0 && !self.is_paused {
                    self.snake.velocity = Vec2::new(1.0, 0.0);
                }
            }
            crossterm::event::KeyCode::Char('p') => {
                self.is_paused = !self.is_paused;
            }
            _ => (),
        }
    }
}

fn main() -> AppResult {
    let mut app = MyApp::default();
    app.run()
}

#[cfg(test)]
mod tests {}
