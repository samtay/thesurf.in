//! Rendering logic for terminals

use super::render::Render;
use super::view::{Color, Content, View};

pub struct Terminal;

impl Render for Terminal {
    type Output = String;

    fn render(view: View) -> Self::Output {
        let mut output = String::new();
        for span in view.spans {
            if let Some(color) = span.style.fg {
                output.push_str(to_code(color));
            }
            match span.content {
                Content::Text(text) => output.push_str(text.as_str()),
                Content::Newline => output.push('\n'),
            }
            if span.style.fg.is_some() {
                output.push_str(ansi::RESET);
            }
        }
        output
    }
}

fn to_code(color: Color) -> &'static str {
    match color {
        Color::Red => ansi::RED,
        Color::Blue => ansi::BLUE,
        Color::Green => ansi::GREEN,
    }
}

mod ansi {
    pub const RED: &str = "\x1B[0;31m";
    pub const GREEN: &str = "\x1B[0;32m";
    pub const BLUE: &str = "\x1B[0;34m";
    pub const RESET: &str = "\x1B[0m";
}
