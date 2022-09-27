mod browser;
mod render;
mod terminal;
mod view;

pub use browser::Browser;
pub use terminal::Terminal;

use render::Render;
use view::View;

pub fn render<R: Render>(view: impl Into<View>) -> R::Output {
    R::render(view.into())
}
