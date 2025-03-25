mod renderer;

use crossterm::style::Color;
use glam::{IVec2, U16Vec2, Vec2};
use renderer::Renderer;
use rgb::Rgb;

use crate::style::{Circle, StyledPrint};

pub struct SimpleCanvas {
    renderer: Renderer,
}

impl SimpleCanvas {
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

    pub fn point(&mut self, pos: IVec2) {
        self.point_with_color(pos, Color::White);
    }

    pub fn point_with_color(&mut self, pos: IVec2, color: Color) {
        if pos.x < 0 || pos.y < 0 {
            return;
        }
        self.draw(pos.as_u16vec2(), Some(color));
    }

    pub fn line(&mut self, start: IVec2, end: IVec2) {
        self.line_with_color(start, end, Color::White);
    }

    pub fn line_with_color(&mut self, start: IVec2, end: IVec2, color: Color) {
        self.draw_line(start, end, Some(color));
    }

    pub fn aa_circle(&mut self, pos: Vec2, circle: Circle) {
        self.draw_aa_circle(pos, circle);
    }

    pub fn aa_line(&mut self, start: Vec2, end: Vec2) {
        self.draw_aa_line(start, end, None);
    }

    pub fn aa_line_with_color(&mut self, start: Vec2, end: Vec2, color: Rgb<u8>) {
        self.draw_aa_line(start, end, Some(color));
    }

    pub fn erase(&mut self, pos: IVec2) {
        if pos.x < 0 || pos.y < 0 {
            return;
        }
        self.draw(pos.as_u16vec2(), None);
    }

    pub fn print<'a>(&mut self, content: impl Into<StyledPrint<'a>>) {
        self.print_styled_content(content.into());
    }

    pub fn at(&self, pos: IVec2) -> Option<Color> {
        if pos.x < 0 || pos.y < 0 {
            return None;
        }
        self.color_at(pos.as_u16vec2())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        assert!(SimpleCanvas::new().is_ok());
    }
}
