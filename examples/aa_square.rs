use clod::{
    style::{CanvasAlignment, Stylize},
    App, AppResult,
};
use crossterm::style::Color;
use glam::{I16Vec2, U16Vec2, Vec2};
use rand::{thread_rng, Rng};

#[derive(Default, Debug)]
struct Entity {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
}

#[derive(Default, Debug)]
struct Circle {
    body: Entity,
    radius: f32,
}

#[derive(Default)]
struct MyApp {
    entities: Vec<Entity>,
    main_body: Circle,
    zoom: f32,
}

impl App for MyApp {
    fn update(&mut self, state: &mut clod::State) -> Result<(), String> {
        let bounds = state.canvas.size();
        self.main_body.body.vel +=
            self.main_body.body.acc * state.delta_seconds() as f32 * self.zoom;
        self.main_body.body.acc = Vec2::ZERO;
        self.main_body.body.vel += Vec2::new(0.0, 9.8) * state.delta_seconds() as f32 * self.zoom;
        self.main_body.body.pos +=
            self.main_body.body.vel * state.delta_seconds() as f32 * self.zoom;
        self.main_body.body.vel.y *= 0.99;
        self.main_body.body.vel.x *= 0.99;
        // self.main_body.acc *= 0.0;

        if self.main_body.body.vel.x.abs() < 0.2 {
            self.main_body.body.vel.x = 0.0;
        }

        if (self.main_body.body.pos.y + self.main_body.radius * self.zoom) >= bounds.y.into() {
            self.main_body.body.vel.y *= -1.0;
            self.main_body.body.pos.y = bounds.y as f32 - self.main_body.radius * self.zoom;
        }

        if (self.main_body.body.pos.x + self.main_body.radius * self.zoom) >= bounds.x.into() {
            self.main_body.body.vel.x *= -1.0;
            self.main_body.body.pos.x = bounds.x as f32 - self.main_body.radius * self.zoom;
        }

        if (self.main_body.body.pos.x - self.main_body.radius * self.zoom) < 0.0 {
            self.main_body.body.vel.x *= -1.0;
            self.main_body.body.pos.x = self.main_body.radius * self.zoom;
        }

        state
            .canvas
            .aa_circle(self.main_body.body.pos, self.main_body.radius * self.zoom);

        state.canvas.print(
            format!(
                "Pos {:.2} : {:.2}, Bounds {} : {}",
                self.main_body.body.pos.x, self.main_body.body.pos.y, bounds.x, bounds.y
            )
            .as_str()
            .align(CanvasAlignment::TOP),
        );
        Ok(())
    }

    fn on_key_event(&mut self, _state: &mut clod::State, event: crossterm::event::KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Char('w') | crossterm::event::KeyCode::Up => {
                self.zoom += 0.1;
            }
            crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Left => {
                self.main_body.body.acc = Vec2::new(-20.0, 0.0) * self.zoom;
            }
            crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Down => {
                self.zoom -= 0.1;
            }
            crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Right => {
                self.main_body.body.acc = Vec2::new(20.0, 0.0) * self.zoom;
            }
            crossterm::event::KeyCode::Char(' ') => {
                self.main_body.body.acc = Vec2::new(0.0, -20.0) * self.zoom;
            }
            _ => {}
        };
    }

    fn init(&mut self, state: &mut clod::State) -> Result<(), String> {
        self.main_body.body.pos = state.canvas.size().as_vec2() / 2.0;
        self.main_body.radius = 1.0;
        self.zoom = 5.0;
        // state
        //     .canvas
        //     .set_background_color(Some(Color::Rgb { r: 0, g: 0, b: 0 }));
        Ok(())
    }
}

fn main() -> AppResult {
    let mut app = MyApp::default();
    app.run()
}

#[cfg(test)]
mod tests {}
