pub const LINE_VERT: &str = "│";
pub const LINE_HORIZONTAL: &str = "─";
pub const CORNER_TOP_LEFT: &str = "┌";
pub const CORNER_TOP_RIGHT: &str = "┐";
pub const CORNER_BTM_LEFT: &str = "└";
pub const CORNER_BTM_RIGHT: &str = "┘";
pub const TEE_LEFT: &str = "┤";
pub const TEE_RIGHT: &str = "├";

/// A view is an AST representing the entire forecast, that can be rendered to
/// different outputs.
pub struct View {
    pub spans: Vec<Span>,
}

/// Internal type synonym to distinguish line breaks on inner widgets
pub type Line = Vec<Span>;

/// A contiguous piece of content with consistent styles. These shouldn't need to
/// nest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub content: Content,
    pub style: Style,
}

impl Span {
    /// Create a new span with default styles
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content: Content::Text(text.into()),
            style: Style::default(),
        }
    }

    /// Create a line break
    pub fn newline() -> Self {
        Self {
            content: Content::Newline,
            style: Style::default(),
        }
    }

    pub fn style(&mut self) -> &mut Style {
        &mut self.style
    }
}

macro_rules! span {
    ($($arg:tt)*) => {{
        let res = Span::new(format!($($arg)*));
        res
    }}
}
pub(super) use span;

/// Content is typically just text in the form of a String. But I think it will
/// make life easier to separate control chars like newlines. So, try not to
/// sneak those into the text values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    Text(String),
    Newline,
}

/// Style attributes that can be added to a given span.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
}

impl Style {
    pub fn fg(&mut self, color: Color) -> &mut Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(&mut self, color: Color) -> &mut Self {
        self.bg = Some(color);
        self
    }

    pub fn bold(&mut self) -> &mut Self {
        self.bold = true;
        self
    }
}

/// The colors available for styling.
// Add more as necessary
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Green,
    Blue,
    Red,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_span_api() {
        let mut span = Span::new("hi");
        span.style().fg(Color::Blue).bg(Color::Red);
        assert_eq!(
            span,
            Span {
                content: Content::Text("hi".to_string()),
                style: Style {
                    fg: Some(Color::Blue),
                    bg: Some(Color::Red),
                    ..Style::default()
                }
            }
        );
    }
}
