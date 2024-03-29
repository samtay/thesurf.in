mod browser;
mod render;
mod terminal;
mod view;

pub use browser::Browser;
pub use render::Render;
pub use terminal::Terminal;
pub use view::{rip::Rip, View};

pub fn render<R: Render>(view: impl Into<View>) -> R::Output {
    R::render(view.into())
}
