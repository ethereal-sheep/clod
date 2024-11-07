mod renderer;

use crossterm::style::Color;
use glam::{U16Vec2, Vec2};
use renderer::Renderer;

use crate::style::StyledPrint;

pub type CanvasPos = U16Vec2;

pub struct Canvas {
    renderer: Renderer,
}

impl Canvas {
    pub fn size(&self) -> U16Vec2 {
        let render_size = self.renderer.size();
        U16Vec2::new(render_size.x, render_size.y * 2)
    }

    pub fn resize(&mut self, size: U16Vec2) {
        self.renderer.resize(size);
    }

    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.renderer.set_background_color(color)
    }

    pub fn draw(&mut self, pos: CanvasPos) {
        self.draw_with_color(pos, Color::White);
    }

    pub fn draw_with_color(&mut self, pos: CanvasPos, color: Color) {
        self.draw_with_some_color(pos, Some(color));
    }

    // pub fn aa(&mut self, pos: Vec2) {
    //     self.aa_with_color(pos, Color::White);
    // }

    // pub fn aa_with_color(&mut self, pos: Vec2, color: Color) {
    //     self.aa_circle_with_some_rgb(pos, None);
    // }

    pub fn aa_circle(&mut self, pos: Vec2, radius: f32) {
        self.aa_circle_with_some_rgb(pos, radius, None);
    }

    pub fn erase(&mut self, pos: CanvasPos) {
        self.draw_with_some_color(pos, None);
    }

    pub fn print<'a>(&mut self, content: impl Into<StyledPrint<'a>>) {
        self.print_styled_content(content.into());
    }

    pub fn at(&self, pos: CanvasPos) -> Option<Color> {
        self.color_at(pos)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        assert!(Canvas::new().is_ok());
    }
}
