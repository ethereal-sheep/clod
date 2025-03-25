use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Color,
};
use engine::SimpleCanvas;
use glam::{IVec2, U16Vec2, Vec2};
use rgb::Rgb;
use style::{Circle, StyledPrint};

mod engine;
pub mod style;

pub struct State {
    canvas: SimpleCanvas,
    quit: bool,
    dt_s: f32,
    elapsed_time_ms: u128,
}

impl State {
    fn new() -> io::Result<Self> {
        Ok(Self {
            canvas: SimpleCanvas::new()?,
            quit: false,
            dt_s: 0.0,
            elapsed_time_ms: 0,
        })
    }

    pub fn exit(&mut self) {
        self.quit = true;
    }

    pub fn delta_seconds(&self) -> f32 {
        self.dt_s
    }

    pub fn elapsed_millis(&self) -> u128 {
        self.elapsed_time_ms
    }

    pub fn canvas_size(&self) -> U16Vec2 {
        self.canvas.size()
    }

    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.canvas.set_background_color(color);
    }

    pub fn point(&mut self, pos: IVec2) {
        self.canvas.point(pos);
    }

    pub fn point_with_color(&mut self, pos: IVec2, color: Color) {
        self.canvas.point_with_color(pos, color);
    }

    pub fn line(&mut self, start: IVec2, end: IVec2) {
        self.canvas.line(start, end);
    }

    pub fn line_with_color(&mut self, start: IVec2, end: IVec2, color: Color) {
        self.canvas.line_with_color(start, end, color);
    }

    pub fn aa_circle(&mut self, pos: Vec2, circle: Circle) {
        self.canvas.aa_circle(pos, circle);
    }

    pub fn aa_line(&mut self, start: Vec2, end: Vec2) {
        self.canvas.aa_line(start, end);
    }

    pub fn aa_line_with_color(&mut self, start: Vec2, end: Vec2, color: Rgb<u8>) {
        self.canvas.aa_line_with_color(start, end, color);
    }

    pub fn erase(&mut self, pos: IVec2) {
        self.canvas.erase(pos);
    }

    pub fn print<'a>(&mut self, content: impl Into<StyledPrint<'a>>) {
        self.canvas.print(content);
    }

    pub fn at(&self, pos: IVec2) -> Option<Color> {
        self.canvas.at(pos)
    }
}

#[derive(Debug)]
pub enum AppError {
    IoError(io::Error),
    InitError(String),
    UpdateError(String),
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

pub type AppResult = Result<(), AppError>;

pub trait App {
    fn update(&mut self, state: &mut State) -> Result<(), String>;
    fn init(&mut self, state: &mut State) -> Result<(), String>;
    fn on_key_event(&mut self, state: &mut State, event: KeyEvent);

    fn run(&mut self) -> AppResult {
        let mut state = State::new()?;
        let global_timer = Instant::now();
        let mut timer = Instant::now();
        if let Err(err) = self.init(&mut state) {
            return Err(AppError::InitError(err));
        }
        while !state.quit {
            let interval: u128 = 16;
            'poll_loop: loop {
                let poll_duration = interval.saturating_sub(timer.elapsed().as_millis());
                if event::poll(Duration::from_millis(poll_duration.try_into().unwrap()))? {
                    let event = event::read()?;
                    match event {
                        Event::FocusGained => todo!(),
                        Event::FocusLost => todo!(),
                        Event::Key(key_event) => {
                            self.on_key_event(&mut state, key_event);
                            match key_event.code {
                                KeyCode::Char('q') => state.exit(),
                                KeyCode::Esc => state.exit(),
                                KeyCode::Char('c') => {
                                    if let KeyModifiers::CONTROL = key_event.modifiers {
                                        state.exit()
                                    }
                                }
                                _ => continue,
                            };
                        }
                        Event::Resize(columns, rows) => {
                            state.canvas.resize(U16Vec2::new(columns, rows))
                        }
                        _ => continue,
                    }
                }
                if timer.elapsed().as_millis() > interval {
                    break 'poll_loop;
                }
            }

            state.dt_s = timer.elapsed().as_secs_f32();
            state.elapsed_time_ms = global_timer.elapsed().as_millis();
            timer = Instant::now();
            if let Err(err) = self.update(&mut state) {
                return Err(AppError::UpdateError(err));
            }
            state.canvas.render()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_seed_from_u64() {}
}
