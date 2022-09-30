//! Rendering logic for browsers

use super::render::Render;
use super::view::{Color, Content, View};

pub struct Browser;

impl Render for Browser {
    type Output = String;

    fn render<V: Into<View>>(view: V) -> Self::Output {
        let mut output = String::new();
        // insert preamble
        output.push_str(
            r#"<html>
                <head>
                    <link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Fira+Code">
                    <style type="text/css">
                        body {
                            background: #282828;
                            color: #ebdbb2;
                        }
                        pre {
                            font-family: "Fira Code", "Courier New", "DejaVu Sans Mono", "Lucida Console", monospace;
                        }
                        .bold {
                            font-weight: bold;
                        }
                        .red {
                            color: #cc241d;
                        }
                        .blue {
                            color: #bbbbbb;
                        }
                        .green {
                            color: #98971a;
                        }
                    </style>
                </head>
                <body><pre>"#,
        );
        for span in view.into().spans {
            if let Some(color) = span.style.fg {
                output.push_str("<span class=\"");
                output.push_str(color_to_str(color));
                output.push_str("\">");
            }
            match span.content {
                Content::Text(text) => output.push_str(text.as_str()),
                Content::Newline => output.push('\n'),
            }
            if span.style.fg.is_some() {
                output.push_str("</span>");
            }
        }
        // close tags
        output.push_str(
            r#"
                </pre></body>
            </html>
        "#,
        );

        output
    }
}

fn color_to_str(color: Color) -> &'static str {
    match color {
        Color::Red => "red",
        Color::Green => "green",
        Color::Blue => "blue",
    }
}
