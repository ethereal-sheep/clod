use bitflags::bitflags;
use crossterm::style::{Attribute, Attributes, Color, ContentStyle};
use glam::{U16Vec2, Vec2};
use paste::paste;

macro_rules! attribute_function {
    (Attribute::$attribute:ident) => {
        paste! {
            calculated_docs! {
                #[doc = concat!(
                    "Applies the [`",
                    stringify!($attribute),
                    "`](Attribute::",
                    stringify!($attribute),
                    ") attribute to the text.",
                )]
                fn [<$attribute:snake>](self) -> Self::Styled {
                    self.attribute(Attribute::$attribute)
                }
            }
        }
    };
}

macro_rules! border_function {
    ($border:ident) => {
        paste! {
            #[doc = concat!(
                "Sets the border color to [`",
                stringify!($color),
                "`](Color::",
                stringify!($color),
                ")."
            )]
            fn [<$border _border_with>](self, color: Color) -> Self::Styled {
                let mut styled = self.stylize();
                styled
                    .as_mut()
                    .border_style
                    .[<$border _border>] = Some(color);
                styled
            }
        }
    };
}

macro_rules! padding_function {
    ($padding:ident) => {
        paste! {
            #[doc = concat!(
                "Sets the padding to [`",
                stringify!($color),
                "`](Color::",
                stringify!($color),
                ")."
            )]
            fn [<$padding _padding>](self, value: u16) -> Self::Styled {
                let mut styled = self.stylize();
                styled
                    .as_mut()
                    .padding
                    .[<$padding _padding>] = value;
                styled
            }
        }
    };
}

macro_rules! color_function {
    (Color::$color:ident) => {
        paste! {
            calculated_docs! {
                #[doc = concat!(
                    "Sets the foreground color to [`",
                    stringify!($color),
                    "`](Color::",
                    stringify!($color),
                    ")."
                )]
                fn [<$color:snake>](self) -> Self::Styled {
                    self.with(Color::$color)
                }

                #[doc = concat!(
                    "Sets the background color to [`",
                    stringify!($color),
                    "`](Color::",
                    stringify!($color),
                    ")."
                )]
                fn [<on_ $color:snake>](self) -> Self::Styled {
                    self.on(Color::$color)
                }

                #[doc = concat!(
                    "Sets the underline color to [`",
                    stringify!($color),
                    "`](Color::",
                    stringify!($color),
                    ")."
                )]
                fn [<underline_ $color:snake>](self) -> Self::Styled {
                    self.underline(Color::$color)
                }

                #[doc = concat!(
                    "Sets the border color to [`",
                    stringify!($color),
                    "`](Color::",
                    stringify!($color),
                    ")."
                )]
                fn [<border_ $color:snake>](self) -> Self::Styled {
                    self.border_with(Color::$color)
                }

                #[doc = concat!(
                    "Sets the border color to [`",
                    stringify!($color),
                    "`](Color::",
                    stringify!($color),
                    ")."
                )]
                fn [<top_border_ $color:snake>](self) -> Self::Styled {
                    self.top_border_with(Color::$color)
                }

                #[doc = concat!(
                    "Sets the border color to [`",
                    stringify!($color),
                    "`](Color::",
                    stringify!($color),
                    ")."
                )]
                fn [<bottom_border_ $color:snake>](self) -> Self::Styled {
                    self.bottom_border_with(Color::$color)
                }

                #[doc = concat!(
                    "Sets the border color to [`",
                    stringify!($color),
                    "`](Color::",
                    stringify!($color),
                    ")."
                )]
                fn [<left_border_ $color:snake>](self) -> Self::Styled {
                    self.left_border_with(Color::$color)
                }

                #[doc = concat!(
                    "Sets the border color to [`",
                    stringify!($color),
                    "`](Color::",
                    stringify!($color),
                    ")."
                )]
                fn [<right_border_ $color:snake>](self) -> Self::Styled {
                    self.right_border_with(Color::$color)
                }
            }
        }
    };
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CanvasAlignment: u8 {
        const CENTER = 0x01;
        const TOP = 0x02;
        const BOTTOM = 0x04;
        const LEFT = 0x08;
        const RIGHT = 0x10;
    }
}

impl CanvasAlignment {
    pub fn apply(&self, canvas_size: U16Vec2) -> CanvasPos {
        let mut current_vec: Option<Vec2> = None;
        let canvas_limit = (canvas_size).as_vec2();
        let half_size = canvas_limit / 2.0;

        if self.contains(CanvasAlignment::TOP) {
            let top = -half_size.with_x(0.0);
            current_vec = current_vec.map(|pos| pos + top).or(Some(top));
        }

        if self.contains(CanvasAlignment::BOTTOM) {
            let bottom = half_size.with_x(0.0);
            current_vec = current_vec.map(|pos| pos + bottom).or(Some(bottom));
        }

        if self.contains(CanvasAlignment::LEFT) {
            let left = -half_size.with_y(0.0);
            current_vec = current_vec.map(|pos| pos + left).or(Some(left));
        }

        if self.contains(CanvasAlignment::RIGHT) {
            let right = half_size.with_y(0.0);
            current_vec = current_vec.map(|pos| pos + right).or(Some(right));
        }

        if self.contains(CanvasAlignment::CENTER) {
            current_vec = current_vec.map(|pos| pos / 2.0)
        }

        (half_size + current_vec.unwrap_or(Vec2::ZERO)).as_u16vec2()
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum BorderType {
    #[default]
    HalfBlock,
    PaddedHalfBlock,
    Line,
}

/// The style that can be put on content.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct BorderStyle {
    /// The border type.
    pub border_type: BorderType,
    /// The top border color
    pub top_border: Option<Color>,
    /// The bottom border color
    pub bottom_border: Option<Color>,
    /// The right border color
    pub right_border: Option<Color>,
    /// The left border color
    pub left_border: Option<Color>,
}

impl BorderStyle {
    pub fn border_width(&self) -> u16 {
        1
    }

    pub fn left_width(&self) -> u16 {
        self.left_border.map_or(0, |_| self.border_width())
    }

    pub fn right_width(&self) -> u16 {
        self.right_border.map_or(0, |_| self.border_width())
    }

    pub fn top_width(&self) -> u16 {
        self.top_border.map_or(0, |_| self.border_width())
    }

    pub fn bottom_width(&self) -> u16 {
        self.bottom_border.map_or(0, |_| self.border_width())
    }

    pub fn extra_width(&self) -> u16 {
        self.left_width() + self.right_width()
    }

    pub fn extra_height(&self) -> u16 {
        self.top_width() + self.bottom_width()
    }
}

/// The style that can be put on content.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Padding {
    /// The top padding
    pub top_padding: u16,
    /// The bottom padding
    pub bottom_padding: u16,
    /// The right padding
    pub right_padding: u16,
    /// The left padding
    pub left_padding: u16,
}

/// The style that can be put on content.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct PrintStyle {
    /// The foreground color.
    pub foreground_color: Option<Color>,
    /// The background color.
    pub background_color: Option<Color>,
    /// The underline color.
    pub underline_color: Option<Color>,
    /// The border style
    pub border_style: BorderStyle,
    /// The content padding
    pub padding: Padding,
    /// The content alignment
    pub alignment: Option<CanvasAlignment>,
    /// List of attributes.
    pub attributes: Attributes,
}

impl PrintStyle {
    /// Creates a `StyledContent` by applying the style to the given `val`.
    #[inline]
    pub fn apply(self, val: &str) -> StyledPrint<'_> {
        StyledPrint::new(val, self)
    }

    pub fn left_width(&self) -> u16 {
        self.padding.left_padding + self.border_style.left_width()
    }

    pub fn right_width(&self) -> u16 {
        self.padding.right_padding + self.border_style.right_width()
    }

    pub fn top_width(&self) -> u16 {
        self.padding.top_padding + self.border_style.top_width()
    }

    pub fn bottom_width(&self) -> u16 {
        self.padding.bottom_padding + self.border_style.bottom_width()
    }

    pub fn extra_width(&self) -> u16 {
        self.left_width() + self.right_width()
    }

    pub fn extra_height(&self) -> u16 {
        self.top_width() + self.bottom_width()
    }

    pub(crate) fn content_style(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: self.foreground_color,
            background_color: self.background_color,
            underline_color: self.underline_color,
            attributes: self.attributes,
        }
    }
}

impl AsRef<PrintStyle> for PrintStyle {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl AsMut<PrintStyle> for PrintStyle {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

/// Provides a set of methods to set attributes and colors.
///
/// # Examples
///
/// ```no_run
/// use crossterm::style::Stylize;
///
/// println!("{}", "Bold text".bold());
/// println!("{}", "Underlined text".underlined());
/// println!("{}", "Negative text".negative());
/// println!("{}", "Red on blue".red().on_blue());
/// ```
pub trait Stylize: Sized {
    /// This type with styles applied.
    type Styled: AsRef<PrintStyle> + AsMut<PrintStyle>;

    /// Styles this type.
    fn stylize(self) -> Self::Styled;

    /// Sets the foreground color.
    fn with(self, color: Color) -> Self::Styled {
        let mut styled = self.stylize();
        styled.as_mut().foreground_color = Some(color);
        styled
    }

    /// Sets the background color.
    fn on(self, color: Color) -> Self::Styled {
        let mut styled = self.stylize();
        styled.as_mut().background_color = Some(color);
        styled
    }

    /// Sets the underline color.
    fn underline(self, color: Color) -> Self::Styled {
        let mut styled = self.stylize();
        styled.as_mut().underline_color = Some(color);
        styled
    }

    /// Styles the content with the attribute.
    fn attribute(self, attr: Attribute) -> Self::Styled {
        let mut styled = self.stylize();
        styled.as_mut().attributes.set(attr);
        styled
    }

    fn align(self, alignment: CanvasAlignment) -> Self::Styled {
        let mut styled = self.stylize();
        styled.as_mut().alignment = Some(alignment);
        styled
    }

    border_function!(top);
    border_function!(bottom);
    border_function!(left);
    border_function!(right);

    /// Sets the foreground color.
    fn border_with(self, color: Color) -> Self::Styled {
        let mut styled = self.stylize();
        let border_style = &mut styled.as_mut().border_style;
        border_style.top_border = Some(color);
        border_style.bottom_border = Some(color);
        border_style.left_border = Some(color);
        border_style.right_border = Some(color);
        styled
    }

    padding_function!(top);
    padding_function!(bottom);
    padding_function!(left);
    padding_function!(right);

    /// Sets a uniform padding.
    fn padding(self, value: u16) -> Self::Styled {
        let mut styled = self.stylize();
        styled.as_mut().padding.top_padding = value;
        styled.as_mut().padding.bottom_padding = value;
        styled.as_mut().padding.left_padding = value;
        styled.as_mut().padding.right_padding = value;
        styled
    }

    /// Sets a uniform horizontal padding.
    fn horizontal_padding(self, value: u16) -> Self::Styled {
        let mut styled = self.stylize();
        styled.as_mut().padding.left_padding = value;
        styled.as_mut().padding.right_padding = value;
        styled
    }
    /// Sets a vertical padding.
    fn vertical_padding(self, value: u16) -> Self::Styled {
        let mut styled = self.stylize();
        styled.as_mut().padding.top_padding = value;
        styled.as_mut().padding.bottom_padding = value;
        styled
    }

    attribute_function!(Attribute::Reset);
    attribute_function!(Attribute::Bold);
    attribute_function!(Attribute::Underlined);
    attribute_function!(Attribute::Reverse);
    attribute_function!(Attribute::Dim);
    attribute_function!(Attribute::Italic);
    attribute_function!(Attribute::SlowBlink);
    attribute_function!(Attribute::RapidBlink);
    attribute_function!(Attribute::Hidden);
    attribute_function!(Attribute::CrossedOut);

    color_function!(Color::Black);
    color_function!(Color::DarkGrey);
    color_function!(Color::Red);
    color_function!(Color::DarkRed);
    color_function!(Color::Green);
    color_function!(Color::DarkGreen);
    color_function!(Color::Yellow);
    color_function!(Color::DarkYellow);
    color_function!(Color::Blue);
    color_function!(Color::DarkBlue);
    color_function!(Color::Magenta);
    color_function!(Color::DarkMagenta);
    color_function!(Color::Cyan);
    color_function!(Color::DarkCyan);
    color_function!(Color::White);
    color_function!(Color::Grey);
}

impl Stylize for PrintStyle {
    type Styled = Self;
    #[inline]
    fn stylize(self) -> Self::Styled {
        self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StyledPrint<'a> {
    /// A content to apply the style on.
    content: &'a str,
    /// The style (colors, content attributes).
    style: PrintStyle,
}

impl<'a> StyledPrint<'a> {
    /// Creates a new `StyledContent`.
    #[inline]
    pub fn new(content: &'a str, style: PrintStyle) -> StyledPrint<'a> {
        StyledPrint { style, content }
    }

    /// Returns the content.
    #[inline]
    pub fn content(&self) -> &'a str {
        self.content
    }

    /// Returns the style.
    #[inline]
    pub fn style(&self) -> &PrintStyle {
        &self.style
    }

    /// Returns a mutable reference to the style, so that it can be further
    /// manipulated
    #[inline]
    pub fn style_mut(&mut self) -> &mut PrintStyle {
        &mut self.style
    }
}

impl AsRef<PrintStyle> for StyledPrint<'_> {
    fn as_ref(&self) -> &PrintStyle {
        &self.style
    }
}
impl AsMut<PrintStyle> for StyledPrint<'_> {
    fn as_mut(&mut self) -> &mut PrintStyle {
        &mut self.style
    }
}

impl<'a> Stylize for StyledPrint<'a> {
    type Styled = StyledPrint<'a>;
    fn stylize(self) -> Self::Styled {
        self
    }
}

impl<'a> Stylize for &'a str {
    type Styled = StyledPrint<'a>;
    fn stylize(self) -> Self::Styled {
        self.into()
    }
}

impl<'a> From<&'a str> for StyledPrint<'a> {
    fn from(val: &'a str) -> Self {
        StyledPrint {
            content: val,
            style: PrintStyle::default(),
        }
    }
}

// Workaround for https://github.com/rust-lang/rust/issues/78835
macro_rules! calculated_docs {
    ($(#[doc = $doc:expr] $item:item)*) => { $(#[doc = $doc] $item)* };
}
// Remove once https://github.com/rust-lang/rust-clippy/issues/7106 stabilizes.
#[allow(clippy::single_component_path_imports)]
#[allow(clippy::useless_attribute)]
use calculated_docs;

use crate::engine::CanvasPos;

#[cfg(test)]
mod tests {
    use crossterm::style::{Attribute, Color};
    use glam::I16Vec2;

    use super::*;

    #[test]
    fn set_fg_bg_add_attr() {
        let style = PrintStyle::default()
            .with(Color::Blue)
            .on(Color::Red)
            .attribute(Attribute::Bold);

        assert_eq!(style.foreground_color, Some(Color::Blue));
        assert_eq!(style.background_color, Some(Color::Red));
        assert!(style.attributes.has(Attribute::Bold));

        let mut styled_content = style.apply("test");

        styled_content = styled_content
            .with(Color::Green)
            .on(Color::Magenta)
            .attribute(Attribute::NoItalic);

        let style = styled_content.style();

        assert_eq!(style.foreground_color, Some(Color::Green));
        assert_eq!(style.background_color, Some(Color::Magenta));
        assert!(style.attributes.has(Attribute::Bold));
        assert!(style.attributes.has(Attribute::NoItalic));
    }

    #[test]
    fn apply_canvas_align() {
        let canvas_size = U16Vec2::new(10, 11);
        assert_eq!(CanvasAlignment::TOP.apply(canvas_size), U16Vec2::new(4, 0));
        assert_eq!(
            CanvasAlignment::BOTTOM.apply(canvas_size),
            U16Vec2::new(4, 10)
        );
        assert_eq!(CanvasAlignment::LEFT.apply(canvas_size), U16Vec2::new(0, 5));
        assert_eq!(
            CanvasAlignment::RIGHT.apply(canvas_size),
            U16Vec2::new(9, 5)
        );
        assert_eq!(
            (CanvasAlignment::TOP | CanvasAlignment::LEFT).apply(canvas_size),
            U16Vec2::ZERO
        );
        assert_eq!(
            (CanvasAlignment::BOTTOM | CanvasAlignment::RIGHT).apply(canvas_size),
            canvas_size.saturating_add_signed(I16Vec2::NEG_ONE)
        );
        assert_eq!(
            (CanvasAlignment::TOP
                | CanvasAlignment::LEFT
                | CanvasAlignment::BOTTOM
                | CanvasAlignment::RIGHT)
                .apply(canvas_size),
            U16Vec2::new(4, 5)
        );
        assert_eq!(
            (CanvasAlignment::TOP
                | CanvasAlignment::LEFT
                | CanvasAlignment::BOTTOM
                | CanvasAlignment::RIGHT)
                .apply(canvas_size),
            CanvasAlignment::CENTER.apply(canvas_size)
        );
    }
}
