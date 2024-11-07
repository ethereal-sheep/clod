use clod::{
    style::{CanvasAlignment, Stylize},
    App, AppResult,
};
use crossterm::style::Color;
use glam::{I16Vec2, U16Vec2};
use rand::{thread_rng, Rng};

struct Entity {
    pos: U16Vec2,
    vel: I16Vec2,
    lives: u8,
    collided: bool,
}

#[derive(Default)]
struct MyApp {
    entities: Vec<Entity>,
}

const MAX_LIVES: u8 = 100;

impl App for MyApp {
    fn update(&mut self, state: &mut clod::State) -> Result<(), String> {
        let bounds = state.canvas.size();
        for entity in self.entities.iter_mut() {
            entity.pos = entity.pos.saturating_add_signed(entity.vel);
        }

        for i in 1..self.entities.len() {
            let (l, r) = self.entities.split_at_mut(i);
            let current = &mut l[l.len() - 1];
            for other in r.iter_mut() {
                if current.pos == other.pos {
                    // same, random dir
                    let deflect_vertically = thread_rng().gen_bool(0.5);
                    if deflect_vertically {
                        current.vel.y *= -1;
                        other.vel.y *= -1;
                    } else {
                        current.vel.x *= -1;
                        other.vel.x *= -1;
                    }
                    current.lives = current.lives.saturating_sub(1);
                    other.lives = other.lives.saturating_sub(1);
                    current.collided = true;
                    other.collided = true;
                } else if current.pos.y == other.pos.y && current.pos.x.abs_diff(other.pos.x) == 1 {
                    current.vel.x *= -1;
                    other.vel.x *= -1;
                    current.lives = current.lives.saturating_sub(1);
                    other.lives = other.lives.saturating_sub(1);
                    current.collided = true;
                    other.collided = true;
                } else if current.pos.x == other.pos.x && current.pos.y.abs_diff(other.pos.y) == 1 {
                    current.vel.y *= -1;
                    other.vel.y *= -1;
                    current.lives = current.lives.saturating_sub(1);
                    other.lives = other.lives.saturating_sub(1);
                    current.collided = true;
                    other.collided = true;
                }
            }
        }

        for entity in self.entities.iter_mut() {
            if entity.pos.y >= bounds.y - 1 || entity.pos.y == 0 {
                entity.vel.y *= -1;
                entity.lives = entity.lives.saturating_sub(1);
            }
            if entity.pos.x >= bounds.x - 1 || entity.pos.x == 0 {
                entity.vel.x *= -1;
                entity.lives = entity.lives.saturating_sub(1);
            }
        }

        for entity in self.entities.iter_mut() {
            if entity.collided {
                state.canvas.draw_with_color(entity.pos, Color::White);
                entity.collided = false;
            } else {
                let saturation = (entity.lives as f32 / MAX_LIVES as f32 * 255.0) as u8;
                state.canvas.draw_with_color(
                    entity.pos,
                    Color::Rgb {
                        r: 255 - saturation,
                        g: saturation,
                        b: 100,
                    },
                );
            }
        }

        self.entities.retain(|e| e.lives > 0);

        state.canvas.print(
            "Press A to spawn entity"
                .align(CanvasAlignment::TOP | CanvasAlignment::LEFT)
                .padding(2),
        );

        Ok(())
    }

    fn on_key_event(&mut self, state: &mut clod::State, event: crossterm::event::KeyEvent) {
        if let crossterm::event::KeyCode::Char('a') = event.code {
            let x = if thread_rng().gen_bool(0.5) { 1 } else { -1 };
            let y = if thread_rng().gen_bool(0.5) { 1 } else { -1 };

            self.entities.push(Entity {
                pos: state.canvas.size() / 2,
                vel: I16Vec2::new(x, y),
                lives: MAX_LIVES,
                collided: false,
            });
        }
    }

    fn init(&mut self, _state: &mut clod::State) -> Result<(), String> {
        Ok(())
    }
}

fn main() -> AppResult {
    let mut app = MyApp::default();
    app.run()
}

#[cfg(test)]
mod tests {}
