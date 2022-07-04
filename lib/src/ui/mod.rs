mod browser;
mod render;
mod terminal;
mod view;

pub use browser::Browser;
pub use terminal::Terminal;

use crate::msw::forecast::Forecast;
use render::Render;
use view::View;

pub fn render<R: Render>(forecast: Vec<Forecast>) -> R::Output {
    let view = View::draw(forecast);
    R::render(view)
}
