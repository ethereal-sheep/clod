use clod::{App, AppResult};
use glam::{I16Vec2, U16Vec2};

struct Entity {
    pos: U16Vec2,
    vel: I16Vec2,
}

#[derive(Default)]
struct MyApp {
    entities: Vec<Entity>,
}

impl App for MyApp {
    fn update(&mut self, state: &mut clod::State) -> Result<(), String> {
        let bounds = state.canvas.size();
        for entity in self.entities.iter_mut() {
            entity.pos = entity.pos.saturating_add_signed(entity.vel);
            if entity.pos.y >= bounds.y - 1 || entity.pos.y == 0 {
                entity.vel.y *= -1;
            }
            if entity.pos.x >= bounds.x - 1 || entity.pos.x == 0 {
                entity.vel.x *= -1;
            }
            state.canvas.draw(entity.pos);
        }

        Ok(())
    }

    fn on_key_event(&mut self, _state: &mut clod::State, event: crossterm::event::KeyEvent) {
        if let crossterm::event::KeyCode::Char('a') = event.code {
            self.entities.push(Entity {
                pos: U16Vec2::new(0, 0),
                vel: I16Vec2::new(1, (self.entities.len() % 4 + 1) as i16),
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
