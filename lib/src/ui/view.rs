use crate::msw::forecast::Forecast;

/// A view is an AST representing the entire forecast, that can be rendered to
/// different outputs.
pub struct View {
    pub spans: Vec<Span>,
}

impl View {
    /// Transform a forecast into stylized text snippets
    pub fn draw(_forecast: Vec<Forecast>) -> Self {
        todo!()
    }
}

/// A contiguous piece of text with consistent styles. These shouldn't need to
/// nest.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

impl Span {
    /// Create a new span with default styles
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }

    /// Create a line break
    pub fn newline() -> Self {
        Self::new("\n")
    }

    pub fn style(&mut self) -> &mut Style {
        &mut self.style
    }
}

/// Style attributes that can be added to a given span.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
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
#[derive(Copy, Clone, Debug, PartialEq)]
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
                text: "hi".to_string(),
                style: Style {
                    fg: Some(Color::Blue),
                    bg: Some(Color::Red),
                    ..Style::default()
                }
            }
        );
    }
}
