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
use glam::{IVec2, U16Vec2, Vec2};
use line_drawing::{Bresenham, XiaolinWu};
use rand_distr::num_traits::pow;
use rgb::Rgb;
use unicode_width::UnicodeWidthStr;

use crate::style::{CanvasAlignment, Circle, StyledPrint};

use super::SimpleCanvas;

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

impl Cell {
    fn with_background_color(color: Option<Color>) -> Self {
        let style = ContentStyle {
            background_color: color,
            ..Default::default()
        };
        Self { c: ' ', style }
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
    default_cell: Option<Cell>,
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
            default_cell: None,
        }
    }

    pub(super) fn resize(&mut self, size: U16Vec2) {
        if self.size == size {
            return;
        }
        self.display.clear();
        self.hidden.clear();
        self.display.resize(
            size.element_product() as usize,
            self.default_cell.clone().unwrap_or_default(),
        );
        self.hidden.resize(
            size.element_product() as usize,
            self.default_cell.clone().unwrap_or_default(),
        );
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
        self.hidden
            .fill(self.default_cell.clone().unwrap_or_default());
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

    pub(super) fn set_default_cell(&mut self, cell: Option<Cell>) {
        self.default_cell = cell
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

    pub(super) fn get_background_color(&self) -> Option<Color> {
        self.buffer
            .default_cell
            .clone()
            .and_then(|cell| cell.style.background_color)
    }

    pub(super) fn set_background_color(&mut self, color: Option<Color>) {
        self.buffer
            .set_default_cell(Some(Cell::with_background_color(color)));
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

impl Circle {
    fn inner_stroke(&self) -> f32 {
        if self.inner_stroke.is_none() && self.outer_stroke.is_none() {
            return 0.5;
        }
        self.inner_stroke.unwrap_or_default()
    }

    fn outer_stroke(&self) -> f32 {
        if self.inner_stroke.is_none() && self.outer_stroke.is_none() {
            return 0.5;
        }
        self.outer_stroke.unwrap_or_default()
    }
}

impl SimpleCanvas {
    pub(crate) fn new() -> io::Result<Self> {
        Ok(Self {
            renderer: Renderer::new()?,
        })
    }

    pub(crate) fn render(&mut self) -> io::Result<()> {
        self.renderer.render()
    }

    pub(super) fn half_block_position_to_rendered_position(&self, pos: U16Vec2) -> Option<U16Vec2> {
        let canvas_size = self.size();
        if pos.x >= canvas_size.x || pos.y >= canvas_size.y {
            return None;
        }

        Some(U16Vec2::new(pos.x, pos.y / 2))
    }

    pub(super) fn draw(&mut self, pos: U16Vec2, color: Option<Color>) {
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

    pub(super) fn draw_aa_circle(&mut self, pos: Vec2, circle: Circle) {
        if circle.radius <= 0.0 {
            return;
        }

        let color = circle.stroke_color.unwrap_or(Rgb {
            r: 255,
            g: 255,
            b: 255,
        });

        let outer_stroke_sq =
            (circle.radius + circle.outer_stroke()) * (circle.radius + circle.outer_stroke());
        let inner_stroke_sq =
            (circle.radius - circle.inner_stroke()) * (circle.radius - circle.inner_stroke());
        let span_vector = Vec2::new(
            circle.radius + circle.outer_stroke(),
            circle.radius + circle.outer_stroke(),
        );

        let top_left = (pos - span_vector + Vec2::ONE / 2.0).floor().as_u16vec2();
        let bottom_right = (pos + span_vector + Vec2::ONE / 2.0).ceil().as_u16vec2();

        for y in top_left.y..bottom_right.y {
            for x in top_left.x..bottom_right.x {
                let canvas_pos = U16Vec2::new(x, y);

                let get_sub_pixel_points = |pow_of_2: usize| {
                    let divisions = pow(2usize, pow_of_2);
                    let step = 1.0 / divisions as f32;
                    let mut points: Vec<Vec2> =
                        Vec::with_capacity((divisions + 1) * (divisions + 1));

                    for i in 0..=divisions {
                        for j in 0..=divisions {
                            let delta =
                                Vec2::new(j as f32 * step, i as f32 * step) - Vec2::ONE / 2.0;
                            points.push(canvas_pos.as_vec2() + delta);
                        }
                    }
                    points
                };

                let pixel_vertices = get_sub_pixel_points(1);
                if pixel_vertices
                    .iter()
                    .all(|p| p.distance_squared(pos) > outer_stroke_sq)
                {
                    // do nothing
                    // self.draw_with_color(canvas_pos, Color::White);
                } else if pixel_vertices
                    .iter()
                    .all(|p| p.distance_squared(pos) < inner_stroke_sq)
                {
                } else if pixel_vertices.iter().all(|p| {
                    let dist_sq = p.distance_squared(pos);
                    dist_sq <= outer_stroke_sq && dist_sq >= inner_stroke_sq
                }) {
                    self.draw(
                        canvas_pos,
                        Some(Color::Rgb {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                        }),
                    );
                } else {
                    // on the edge
                    let sub_pixel_vertices = get_sub_pixel_points(2);
                    let count = sub_pixel_vertices
                        .iter()
                        .filter(|p| {
                            let dist_sq = p.distance_squared(pos);
                            dist_sq <= outer_stroke_sq && dist_sq >= inner_stroke_sq
                        })
                        .count() as f32;

                    let magnitude = count / sub_pixel_vertices.len() as f32;
                    // let sin_magnitude = (magnitude * PI).sin();
                    let lerp = |l: f32, r: f32, v: f32| l + (r - l) * v;
                    let background_color = self.background_rgb_at_or_default(canvas_pos);
                    self.draw(
                        canvas_pos,
                        Some(Color::Rgb {
                            r: lerp(background_color.r.into(), color.r.into(), magnitude) as u8,
                            g: lerp(background_color.g.into(), color.g.into(), magnitude) as u8,
                            b: lerp(background_color.b.into(), color.b.into(), magnitude) as u8,
                        }),
                    );
                }
            }
        }
    }

    pub(super) fn draw_line(&mut self, start: IVec2, end: IVec2, color: Option<Color>) {
        for (x, y) in Bresenham::new(
            (start.x, start.y),
            (end.x, end.y),
        ) {
            if x < 0 || y < 0 {
                continue;
            }
            let canvas_pos = U16Vec2::new(x as u16, y as u16);
            self.draw(canvas_pos, color);
        }
    }

    pub(super) fn draw_aa_line(&mut self, start: Vec2, end: Vec2, color: Option<Rgb<u8>>) {
        let lerp = |l: f32, r: f32, v: f32| l + (r - l) * v;

        let color = color.unwrap_or(Rgb {
            r: 255,
            g: 255,
            b: 255,
        });

        for ((x, y), magnitude) in XiaolinWu::<f32, i32>::new((start.x, start.y), (end.x, end.y)) {
            let canvas_pos = U16Vec2::new(x as u16, y as u16);
            let background_color = self.background_rgb_at_or_default(canvas_pos);
            self.draw(
                canvas_pos,
                Some(Color::Rgb {
                    r: lerp(background_color.r.into(), color.r.into(), magnitude) as u8,
                    g: lerp(background_color.g.into(), color.g.into(), magnitude) as u8,
                    b: lerp(background_color.b.into(), color.b.into(), magnitude) as u8,
                }),
            );
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
                    self.draw(U16Vec2::new(x, y), Some(color));
                }
            }
        }

        for y in box_start_y..box_end_y {
            for x in box_start_x..box_end_x {
                if let Some(color) = content.style().border_style.left_border {
                    if x == box_start_x {
                        self.draw(U16Vec2::new(x, y), Some(color));
                    }
                }
                if let Some(color) = content.style().border_style.right_border {
                    if x == box_end_x - 1 {
                        self.draw(U16Vec2::new(x, y), Some(color));
                    }
                }
                if let Some(color) = content.style().border_style.top_border {
                    if y == box_start_y {
                        self.draw(U16Vec2::new(x, y), Some(color));
                    }
                }
                if let Some(color) = content.style().border_style.bottom_border {
                    if y == box_end_y - 1 {
                        self.draw(U16Vec2::new(x, y), Some(color));
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

    pub(super) fn color_at(&self, pos: U16Vec2) -> Option<Color> {
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

    fn background_rgb_at_or_default(&self, pos: U16Vec2) -> Rgb<u8> {
        self.color_at(pos)
            .or_else(|| self.renderer.get_background_color())
            .and_then(|color| {
                if let Color::Rgb { r, g, b } = color {
                    Some(Rgb::new(r, g, b))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| Rgb::new(0, 0, 0))
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
