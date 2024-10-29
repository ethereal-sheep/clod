use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use engine::Canvas;
use glam::U16Vec2;

pub mod engine;
pub mod style;

pub struct State {
    pub canvas: Canvas,
    quit: bool,
    dt_s: f64,
    elapsed_time_ms: u128,
}

impl State {
    fn new() -> io::Result<Self> {
        Ok(Self {
            canvas: Canvas::new()?,
            quit: false,
            dt_s: 0.0,
            elapsed_time_ms: 0,
        })
    }

    pub fn exit(&mut self) {
        self.quit = true;
    }

    pub fn delta_seconds(&self) -> f64 {
        self.dt_s
    }

    pub fn elapsed_millis(&self) -> u128 {
        self.elapsed_time_ms
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
            let interval = 16;
            while timer.elapsed().as_millis() < interval {
                let poll_duration = interval - timer.elapsed().as_millis();
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
            }
            state.dt_s = timer.elapsed().as_secs_f64();
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
