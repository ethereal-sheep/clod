use clod::{
    style::{CanvasAlignment, CircleLike, Stylize},
    App, AppResult,
};
use crossterm::style::Color;
use glam::Vec2;
use rgb::Rgb;

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
    main_body: Circle,
    zoom: f32,
    counter: i32,
}

impl App for MyApp {
    fn update(&mut self, state: &mut clod::State) -> Result<(), String> {
        let bounds = state.canvas_size().as_vec2() - Vec2::ONE;
        self.main_body.body.vel += self.main_body.body.acc * self.zoom;
        self.main_body.body.vel += Vec2::new(0.0, 9.8) * state.delta_seconds() * self.zoom;
        self.main_body.body.pos +=
            self.main_body.body.vel * state.delta_seconds() * self.zoom;
        self.main_body.body.vel.y *= 0.99;
        self.main_body.body.vel.x *= 0.99;
        // self.main_body.acc *= 0.0;

        if self.main_body.body.vel.x.abs() < 0.2 {
            self.main_body.body.vel.x = 0.0;
        }

        if (self.main_body.body.pos.y + self.main_body.radius * self.zoom) >= bounds.y {
            self.main_body.body.vel.y *= -1.0;
            self.main_body.body.pos.y = bounds.y - self.main_body.radius * self.zoom;
            self.main_body.body.vel.x *= 0.98;
        }

        if (self.main_body.body.pos.x + self.main_body.radius * self.zoom) >= bounds.x {
            self.main_body.body.vel.x *= -1.0;
            self.main_body.body.pos.x = bounds.x - self.main_body.radius * self.zoom;
        }

        if (self.main_body.body.pos.x - self.main_body.radius * self.zoom) < 0.0 {
            self.main_body.body.vel.x *= -1.0;
            self.main_body.body.pos.x = self.main_body.radius * self.zoom;
        }

        state.aa_line(self.main_body.body.pos, Vec2::ZERO);
        state.aa_line(self.main_body.body.pos, bounds);
        state.aa_line(bounds - self.main_body.body.pos, bounds);
        state.aa_line_with_color(
            Vec2::ZERO,
            bounds - self.main_body.body.pos,
            Rgb {
                r: 200,
                g: 100,
                b: 200,
            },
        );

        state.aa_circle(
            self.main_body.body.pos,
            clod::style::Circle::with_radius(self.main_body.radius * self.zoom)
                .stroke(3.0)
                .stroke_color(Rgb::new(180, 123, 43)),
        );

        state.print(
            format!(
                "Pos {:.2} : {:.2}, Vel {:.2} : {:.2}, Acc {:.2} : {:.2}, Bounds {} : {}, Counter: {}, dt: {:.0}",
                self.main_body.body.pos.x,
                self.main_body.body.pos.y,
                self.main_body.body.vel.x,
                self.main_body.body.vel.y,
                self.main_body.body.acc.x,
                self.main_body.body.acc.y,
                bounds.x,
                bounds.y,
                self.counter,
                1.0 / state.delta_seconds()
            )
            .as_str()
            .align(CanvasAlignment::TOP),
        );
        self.main_body.body.acc = Vec2::ZERO;
        Ok(())
    }

    fn on_key_event(&mut self, _state: &mut clod::State, event: crossterm::event::KeyEvent) {
        match event.code {
            crossterm::event::KeyCode::Char('w') | crossterm::event::KeyCode::Up => {
                self.zoom += 0.1;
            }
            crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Left => {
                self.main_body.body.acc = Vec2::new(-2.0, 0.0) * self.zoom;
            }
            crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Down => {
                self.zoom -= 0.1;
            }
            crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Right => {
                self.main_body.body.acc = Vec2::new(2.0, 0.0) * self.zoom;
            }
            crossterm::event::KeyCode::Char(' ') => {
                self.main_body.body.acc = Vec2::new(0.0, -2.0) * self.zoom;
                self.counter += 1;
            }
            _ => {}
        };
    }

    fn init(&mut self, state: &mut clod::State) -> Result<(), String> {
        self.main_body.body.pos = state.canvas_size().as_vec2() / 2.0;
        self.main_body.radius = 1.0;
        self.zoom = 5.0;
        state.set_background_color(Some(Color::Rgb { r: 0, g: 0, b: 0 }));
        Ok(())
    }
}

fn main() -> AppResult {
    let mut app = MyApp::default();
    app.run()
}

#[cfg(test)]
mod tests {}
