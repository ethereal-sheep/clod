use std::{
    io::{self, stdout, Stdout, Write},
    mem::swap,
    panic::{set_hook, take_hook},
};

use crossterm::{
    cursor, execute,
    style::{Color, ContentStyle},
    terminal, QueueableCommand,
};
use glam::U16Vec2;
use unicode_width::UnicodeWidthStr;

use crate::style::{CanvasAlignment, StyledPrint};

use super::{Canvas, CanvasPos};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Cell {
    pub(crate) c: char,
    pub(crate) style: ContentStyle,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            style: ContentStyle::default(),
        }
    }
}

pub(super) struct BlockCell<'a> {
    cell: &'a Cell,
}

impl<'a, 'b> BlockCell<'b> {
    pub(super) fn wrap(cell: &'a Cell) -> BlockCell<'b>
    where
        'a: 'b,
    {
        Self { cell }
    }

    pub(super) fn at_top(&self) -> Option<Color> {
        match self.cell.c {
            ' ' => None,
            '▀' | '█' => self.cell.style.foreground_color,
            '▄' => self.cell.style.background_color,
            _ => None,
        }
    }

    pub(super) fn at_bottom(&self) -> Option<Color> {
        match self.cell.c {
            ' ' => None,
            '▄' | '█' => self.cell.style.foreground_color,
            '▀' => self.cell.style.background_color,
            _ => None,
        }
    }
}

pub(super) struct BlockCellMut<'a> {
    cell: &'a mut Cell,
}

impl<'a, 'b> BlockCellMut<'b> {
    pub(super) fn wrap(cell: &'a mut Cell) -> BlockCellMut<'b>
    where
        'a: 'b,
    {
        Self { cell }
    }

    pub(super) fn set_top(&mut self, color: Option<Color>) {
        if color.is_none() {
            return self.unset_top();
        }

        match self.cell.c {
            ' ' => {
                self.cell.c = '▀';
                self.cell.style.foreground_color = color;
            }
            '▀' => self.cell.style.foreground_color = color,
            '▄' => self.cell.style.background_color = color,
            '█' => {
                self.cell.c = '▄';
                self.cell.style.background_color = color;
            }
            _ => {
                self.cell.c = '▀';
                self.cell.style.foreground_color = color;
                self.cell.style.background_color = None;
            }
        }
    }

    pub(super) fn set_bottom(&mut self, color: Option<Color>) {
        if color.is_none() {
            return self.unset_bottom();
        }

        match self.cell.c {
            ' ' => {
                self.cell.c = '▄';
                self.cell.style.foreground_color = color;
            }
            '▀' => self.cell.style.background_color = color,
            '▄' => self.cell.style.foreground_color = color,
            '█' => {
                self.cell.c = '▀';
                self.cell.style.background_color = color;
            }
            _ => {
                self.cell.c = '▄';
                self.cell.style.foreground_color = color;
                self.cell.style.background_color = None;
            }
        }
    }

    pub(super) fn unset_top(&mut self) {
        match self.cell.c {
            ' ' => {
                if let Some(color) = self.cell.style.background_color {
                    self.cell.c = '▄';
                    self.cell.style.foreground_color = Some(color);
                    self.cell.style.background_color = None;
                }
            }
            '▀' => {
                if let Some(color) = self.cell.style.background_color {
                    self.cell.c = '▄';
                    self.cell.style.foreground_color = Some(color);
                    self.cell.style.background_color = None;
                } else {
                    self.cell.c = ' ';
                    self.cell.style.foreground_color = None;
                }
            }
            '▄' => self.cell.style.background_color = None,
            '█' => {
                self.cell.c = '▄';
                self.cell.style.background_color = None;
            }
            _ => {
                self.cell.c = ' ';
                self.cell.style.foreground_color = None;
                self.cell.style.background_color = None;
            }
        }
    }

    fn unset_bottom(&mut self) {
        match self.cell.c {
            ' ' => {
                if let Some(color) = self.cell.style.background_color {
                    self.cell.c = '▀';
                    self.cell.style.foreground_color = Some(color);
                    self.cell.style.background_color = None;
                }
            }
            '▀' => self.cell.style.background_color = None,
            '▄' => {
                if let Some(color) = self.cell.style.background_color {
                    self.cell.c = '▀';
                    self.cell.style.foreground_color = Some(color);
                    self.cell.style.background_color = None;
                } else {
                    self.cell.c = ' ';
                    self.cell.style.foreground_color = None;
                }
            }
            '█' => {
                self.cell.c = '▀';
                self.cell.style.background_color = None;
            }
            _ => {
                self.cell.c = ' ';
                self.cell.style.foreground_color = None;
                self.cell.style.background_color = None;
            }
        }
    }
}

pub(super) struct DoubleBuffer {
    display: Vec<Cell>,
    hidden: Vec<Cell>,
    size: U16Vec2,
}

impl DoubleBuffer {
    pub(super) fn from_values(height: u16, width: u16) -> Self {
        let size = U16Vec2::new(width, height);
        Self::from_size(size)
    }

    pub(super) fn from_size(size: U16Vec2) -> Self {
        Self {
            display: vec![Cell::default(); size.element_product() as usize],
            hidden: vec![Cell::default(); size.element_product() as usize],
            size,
        }
    }

    pub(super) fn resize(&mut self, size: U16Vec2) {
        if self.size == size {
            return;
        }
        self.display.clear();
        self.hidden.clear();
        self.display
            .resize(size.element_product() as usize, Cell::default());
        self.hidden
            .resize(size.element_product() as usize, Cell::default());
        self.size = size;
    }

    pub(super) fn diff(&self, redraw: bool) -> Vec<(&Cell, U16Vec2)> {
        let mut diff = vec![];
        for i in 0..self.len() {
            if redraw || self.hidden.get(i) != self.display.get(i) {
                diff.push((self.hidden.get(i).unwrap(), self.index_to_position(i)));
            }
        }
        diff
    }

    pub(super) fn swap(&mut self) {
        swap(&mut self.hidden, &mut self.display);
        self.hidden.fill(Cell::default());
    }

    pub(super) fn size(&self) -> U16Vec2 {
        self.size
    }

    pub(super) fn bounds(&self, position: &U16Vec2) -> bool {
        self.size.x > position.x && self.size.y > position.y
    }

    pub(super) fn at(&self, normalized_position: U16Vec2) -> Option<&Cell> {
        if !self.bounds(&normalized_position) {
            return None;
        }
        let idx: usize = self.position_to_index(&normalized_position);
        self.get(idx)
    }

    pub(super) fn at_mut(&mut self, normalized_position: U16Vec2) -> Option<&mut Cell> {
        if !self.bounds(&normalized_position) {
            return None;
        }
        let idx: usize = self.position_to_index(&normalized_position);
        self.get_mut(idx)
    }

    fn index_to_position(&self, idx: usize) -> U16Vec2 {
        U16Vec2::new(idx as u16 % self.size.x, idx as u16 / self.size.x)
    }
    fn position_to_index(&self, pos: &U16Vec2) -> usize {
        (self.size.x * pos.y + pos.x).into()
    }

    fn len(&self) -> usize {
        self.hidden.len()
    }

    fn get(&self, index: usize) -> Option<&Cell> {
        self.hidden.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Cell> {
        self.hidden.get_mut(index)
    }
}

pub(super) struct Renderer {
    buffer: DoubleBuffer,
    redraw: bool,
}

impl Renderer {
    pub(crate) fn new() -> io::Result<Self> {
        let (cols, rows) = terminal::size()?;
        let new = Self {
            buffer: DoubleBuffer::from_values(rows, cols),
            redraw: false,
        };
        Self::init()?;
        Ok(new)
    }

    pub(crate) fn render(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        stdout.queue(crossterm::style::ResetColor)?;
        let mut style = ContentStyle::default();

        let diff = self.buffer.diff(self.redraw);
        for (cell, pos) in diff {
            stdout.queue(crossterm::cursor::MoveTo(pos.x, pos.y))?;

            if style != cell.style {
                style = Self::set_terminal_styling(&mut stdout, &style, &cell.style)?;
            }

            stdout.queue(crossterm::style::Print(cell.c))?;
        }

        stdout.flush()?;
        self.redraw = false;
        self.buffer.swap();
        Ok(())
    }

    pub(crate) fn resize(&mut self, size: U16Vec2) {
        self.buffer.resize(size);
        self.redraw = true;
    }

    pub(super) fn size(&self) -> U16Vec2 {
        self.buffer.size()
    }

    pub(super) fn set_terminal_styling(
        stdout: &mut Stdout,
        style: &ContentStyle,
        new: &ContentStyle,
    ) -> io::Result<ContentStyle> {
        if style.background_color != new.background_color {
            match new.background_color {
                Some(x) => {
                    stdout.queue(crossterm::style::SetBackgroundColor(x))?;
                }
                None => {
                    stdout.queue(crossterm::style::SetBackgroundColor(
                        crossterm::style::Color::Reset,
                    ))?;
                }
            }
        }
        if style.foreground_color != new.foreground_color {
            match new.foreground_color {
                Some(x) => {
                    stdout.queue(crossterm::style::SetForegroundColor(x))?;
                }
                None => {
                    stdout.queue(crossterm::style::SetForegroundColor(
                        crossterm::style::Color::Reset,
                    ))?;
                }
            }
        }
        if style.attributes != new.attributes {
            stdout.queue(crossterm::style::SetAttribute(
                crossterm::style::Attribute::Reset,
            ))?;
            if let Some(x) = new.foreground_color {
                stdout.queue(crossterm::style::SetForegroundColor(x))?;
            }
            if let Some(x) = new.background_color {
                stdout.queue(crossterm::style::SetBackgroundColor(x))?;
            }
            stdout.queue(crossterm::style::SetAttributes(new.attributes))?;
        }
        Ok(*new)
    }

    pub(super) fn init() -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), cursor::Hide, terminal::EnterAlternateScreen)?;
        let original_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            // intentionally ignore errors here since we're already in a panic
            let _ = Self::shutdown();
            original_hook(panic_info);
        }));
        Ok(())
    }

    pub(super) fn shutdown() -> io::Result<()> {
        execute!(stdout(), cursor::Show, terminal::LeaveAlternateScreen,)?;
        terminal::disable_raw_mode()
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = Self::shutdown();
    }
}

impl Canvas {
    pub(crate) fn new() -> io::Result<Self> {
        Ok(Self {
            renderer: Renderer::new()?,
        })
    }

    pub(crate) fn render(&mut self) -> io::Result<()> {
        self.renderer.render()
    }

    pub(super) fn half_block_position_to_rendered_position(
        &self,
        pos: CanvasPos,
    ) -> Option<U16Vec2> {
        let canvas_size = self.size();
        if pos.x >= canvas_size.x || pos.y >= canvas_size.y {
            return None;
        }

        Some(U16Vec2::new(pos.x, pos.y / 2))
    }

    pub(super) fn draw_with_some_color(&mut self, pos: CanvasPos, color: Option<Color>) {
        if let Some(mut cell) = self
            .half_block_position_to_rendered_position(pos)
            .and_then(|pos| self.renderer.buffer.at_mut(pos))
            .map(BlockCellMut::wrap)
        {
            if pos.y % 2 == 0 {
                cell.set_top(color);
            } else {
                cell.set_bottom(color);
            }
        }
    }

    pub(super) fn print_styled_content(&mut self, content: StyledPrint<'_>) {
        let style = content.style();
        let content_width = content.content().width() as u16;
        let content_height = if content_width == 0 { 0 } else { 1 };
        let total_width = content_width + style.extra_width();
        let total_height = content_height + style.extra_height();

        let size = self.renderer.size();
        let alignment = content.style().alignment.unwrap_or(CanvasAlignment::CENTER);

        let print_pos = alignment.apply(size);

        let end_x = (print_pos.x + (total_width + 1) / 2).min(size.x);
        let start_x = end_x.saturating_sub(total_width);

        let end_y = (print_pos.y + (total_height + 1) / 2).min(size.y);
        let start_y = end_y.saturating_sub(total_height);

        let line_start_x = start_x + style.left_width();
        let line_start_y = start_y + (style.top_width() + 1) / 2;

        let canvas_start_x = line_start_x;
        let canvas_start_y = line_start_y * 2;

        let canvas_end_x = canvas_start_x + content_width;
        let canvas_end_y = canvas_start_y + content_height * 2;

        let box_start_x = canvas_start_x.saturating_sub(style.left_width());
        let box_start_y = canvas_start_y.saturating_sub(style.top_width());

        let box_end_x = canvas_end_x + style.right_width();
        let box_end_y = canvas_end_y + style.bottom_width();

        for y in box_start_y..box_end_y {
            for x in box_start_x..box_end_x {
                if let Some(color) = content.style().background_color {
                    self.draw_with_color(CanvasPos::new(x, y), color);
                }
            }
        }

        for y in box_start_y..box_end_y {
            for x in box_start_x..box_end_x {
                if let Some(color) = content.style().border_style.left_border {
                    if x == box_start_x {
                        self.draw_with_color(CanvasPos::new(x, y), color);
                    }
                }
                if let Some(color) = content.style().border_style.right_border {
                    if x == box_end_x - 1 {
                        self.draw_with_color(CanvasPos::new(x, y), color);
                    }
                }
                if let Some(color) = content.style().border_style.top_border {
                    if y == box_start_y {
                        self.draw_with_color(CanvasPos::new(x, y), color);
                    }
                }
                if let Some(color) = content.style().border_style.bottom_border {
                    if y == box_end_y - 1 {
                        self.draw_with_color(CanvasPos::new(x, y), color);
                    }
                }
            }
        }

        // write content
        for (i, c) in content.content().chars().enumerate() {
            if let Some(cell) = self
                .renderer
                .buffer
                .at_mut(U16Vec2::new(i as u16 + line_start_x, line_start_y))
            {
                cell.c = c;
                cell.style = content.style().content_style();
            }
        }
    }

    pub(super) fn color_at(&self, pos: CanvasPos) -> Option<Color> {
        if let Some(cell) = self
            .half_block_position_to_rendered_position(pos)
            .and_then(|pos| self.renderer.buffer.at(pos))
            .map(BlockCell::wrap)
        {
            if pos.y % 2 == 0 {
                cell.at_top()
            } else {
                cell.at_bottom()
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn position_to_index() {
        let size = U16Vec2::new(10, 20);
        let buffer = DoubleBuffer::from_size(size);
        assert_eq!(buffer.position_to_index(&U16Vec2::new(0, 0)), 0);
        assert_eq!(buffer.position_to_index(&U16Vec2::new(1, 0)), 1);
        assert_eq!(buffer.position_to_index(&U16Vec2::new(0, 1)), 10);
        assert_eq!(buffer.position_to_index(&U16Vec2::new(5, 7)), 75);
    }

    #[test]
    fn index_to_position() {
        let size = U16Vec2::new(10, 20);
        let buffer = DoubleBuffer::from_size(size);
        assert_eq!(buffer.index_to_position(0), U16Vec2::new(0, 0));
        assert_eq!(buffer.index_to_position(10), U16Vec2::new(0, 1));
        assert_eq!(buffer.index_to_position(1), U16Vec2::new(1, 0));
        assert_eq!(buffer.index_to_position(1), U16Vec2::new(1, 0));
        assert_eq!(buffer.index_to_position(75), U16Vec2::new(5, 7));
    }

    #[test]
    fn render() {
        let mut renderer = Renderer::new().unwrap();
        assert!(renderer.render().is_ok());
    }
}
