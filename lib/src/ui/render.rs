use super::view::View;

/// Specific way in which a given UI will render a `View`.
pub trait Render {
    // Default to string? TBD if this is necessary, there may be some HTML
    // specific type
    type Output;

    fn render(view: View) -> Self::Output;
}
