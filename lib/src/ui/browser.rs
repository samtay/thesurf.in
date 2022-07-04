//! Rendering logic for browsers

use super::render::Render;
use super::view::View;

pub struct Browser;

impl Render for Browser {
    type Output = String;

    fn render(_view: View) -> Self::Output {
        todo!()
    }
}
