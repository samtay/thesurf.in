//! Rendering logic for terminals

use super::render::Render;
use super::view::View;

pub struct Terminal;

impl Render for Terminal {
    type Output = String;

    fn render(_view: View) -> Self::Output {
        todo!()
    }
}
