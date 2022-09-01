use std::cmp::Ordering;

use chrono::Timelike;
use itertools::Itertools;

use crate::msw::forecast::Forecast;

/// Total width of the viewpoint output. If the user's viewpoint is smaller than
/// this, output will look choppy, so we want to minimize it while keeping the
/// view useful.
const VIEWPOINT_WIDTH: usize = 88;
/// Viewpoint width minus the border chars
const INTERIOR_VIEWPOINT_WIDTH: usize = VIEWPOINT_WIDTH - 2;

const LINE_VERT: &str = "│";
const LINE_HORIZONTAL: &str = "─";
const CORNER_TOP_LEFT: &str = "┌";
const CORNER_TOP_RIGHT: &str = "┐";
const CORNER_BTM_LEFT: &str = "└";
const CORNER_BTM_RIGHT: &str = "┘";
const TEE_LEFT: &str = "┤";
const TEE_RIGHT: &str = "├";

/// A view is an AST representing the entire forecast, that can be rendered to
/// different outputs.
pub struct View {
    pub spans: Vec<Span>,
}

impl View {
    /// Transform a forecast into stylized text snippets
    pub fn draw(forecast: Vec<Forecast>) -> Self {
        let mut spans = Vec::new();
        // Graph is uninteresting by day, so make it the full week
        spans.extend(Graph::new(forecast.as_slice()).draw());

        // This may be fragile; assumes 12am,3,6,9,12,3,6,9pm for each day
        // Could probably partition by datetime.day value
        let days = forecast.split_inclusive(|fc| fc.local_timestamp.time().hour() == 21);
        for fc in days {
            spans.push(Span::newline());
            spans.extend(Day::new(fc).draw());
        }
        Self { spans }
    }
}

/// TODO maybe move base stuff to internal mod
// Also: make a more elegant abstraction than this?
trait Border {
    /// Title string (unpadded)
    fn title(&self) -> String;

    /// Wrap inner view with a titled border.
    fn border(&self, inner: Vec<Line>) -> Vec<Span> {
        // Top border of the view manually handles border offsets
        let mut spans = self.border_top();
        spans.push(Span::newline());

        // Wrap border around each interior line
        let border_wrap = [Span::new(LINE_VERT), Span::newline(), Span::new(LINE_VERT)];
        spans.push(Span::new(LINE_VERT));
        spans.extend(inner.as_slice().join(&border_wrap[..]));
        spans.push(Span::new(LINE_VERT));
        spans.push(Span::newline());

        // Bottom border of the view manually handles border offsets
        spans.extend(self.border_bottom());

        spans
    }

    /// Render border top title
    fn border_top(&self) -> Vec<Span> {
        let title = format!(" {} ", self.title());
        // top line
        let box_top = format!(
            "{CORNER_TOP_LEFT}{:─^width$}{CORNER_TOP_RIGHT}",
            "",
            width = title.len()
        );
        let top = format!("{:^width$}", box_top, width = VIEWPOINT_WIDTH);
        // middle line
        let box_mid = format!("{TEE_LEFT}{title}{TEE_RIGHT}");
        let mid = format!(
            "{CORNER_TOP_LEFT}{:─^width$}{CORNER_TOP_RIGHT}",
            box_mid,
            width = INTERIOR_VIEWPOINT_WIDTH
        );
        // bottom line
        let box_btm = format!(
            "{CORNER_BTM_LEFT}{:─^width$}{CORNER_BTM_RIGHT}",
            "",
            width = title.len()
        );
        let btm = format!(
            "{LINE_VERT}{:^width$}{LINE_VERT}",
            box_btm,
            width = INTERIOR_VIEWPOINT_WIDTH
        );
        vec![
            Span::new(top),
            Span::newline(),
            Span::new(mid),
            Span::newline(),
            Span::new(btm),
        ]
    }

    /// Closing for the bottom of the border box
    fn border_bottom(&self) -> Vec<Span> {
        vec![Span::new(format!(
            "{CORNER_BTM_LEFT}{:─^width$}{CORNER_BTM_RIGHT}",
            "",
            width = INTERIOR_VIEWPOINT_WIDTH
        ))]
    }
}

/// The swell graph over a multi-day forecast
struct Graph<'a> {
    forecast: &'a [Forecast],
    min_swell_height: u16,
    max_swell_height: u16,
    midnight: &'a Forecast,
}

impl<'a> Border for Graph<'_> {
    /// Gen title for the week's swell graph
    fn title(&self) -> String {
        let date_init = self
            .forecast
            .iter()
            .map(|fc| fc.local_timestamp)
            .min()
            .unwrap()
            .format("%a %b %d");
        let date_end = self
            .forecast
            .iter()
            .map(|fc| fc.local_timestamp)
            .max()
            .unwrap()
            .format("%a %b %d");
        format!("{date_init} - {date_end}")
    }
}

impl<'a> Graph<'a> {
    const SWELL_GRAPH_HEIGHT: usize = 10;

    /// Panics on empty forecast; TODO handle this above
    // TODO might make drawing easier to do more setup here
    fn new(forecast: &'a [Forecast]) -> Self {
        // TODO some smartness for a good graph range.
        let min_swell_height = 0;
        let max_swell_height =
            // 10.max(
            forecast
                .iter()
                .map(|fc| fc.swell.max_breaking_height)
                .min()
                .unwrap_or(8)
                + 2;
        let midnight = forecast.first().unwrap();
        Self {
            forecast,
            min_swell_height,
            max_swell_height,
            midnight,
        }
    }

    fn draw(self) -> Vec<Span> {
        self.border(self.swell_graph())
    }

    /// Generate the swell graph within the border box
    fn swell_graph(&self) -> Vec<Line> {
        let (legend_bin, legend_width) = self.legend_column();
        let num_bins = self.forecast.len();
        let num_bin_boundaries = num_bins - 1;
        let bin_width = (INTERIOR_VIEWPOINT_WIDTH - legend_width - num_bin_boundaries) / num_bins;
        let used_space = legend_width + num_bin_boundaries + num_bins * bin_width;
        let right_margin = INTERIOR_VIEWPOINT_WIDTH - used_space;

        // Initialize with blank spans of the correct width
        let mut bins = vec![
            vec![Span::new(format!("{:width$}", "", width = bin_width)); num_bins];
            Self::SWELL_GRAPH_HEIGHT
        ];
        let mut boundaries =
            vec![vec![Span::new(" "); num_bin_boundaries]; Self::SWELL_GRAPH_HEIGHT];

        let mut last_height = None;
        for x in 0..num_bins {
            let fc = &self.forecast[x];
            // TODO height is reversed; maybe assemble graph bottom up?
            let height = Self::SWELL_GRAPH_HEIGHT - self.scale(fc.swell.abs_max_breaking_height);
            let color = Self::color(fc);

            // Fill in bin
            for (y, bin_line) in bins.iter_mut().enumerate() {
                let span = &mut bin_line[x];
                match height.cmp(&y) {
                    Ordering::Equal => {
                        *span = Span::new(format!("{:─^width$}", "", width = bin_width))
                    }
                    Ordering::Less => {
                        *span = Span::new(format!("{:.^width$}", "", width = bin_width))
                    }
                    _ => {}
                };
                span.style().fg(color);
            }
            // Fill in left-side boundary
            if let Some(last_height) = last_height {
                for (y, boundary_line) in boundaries.iter_mut().enumerate() {
                    let span = &mut boundary_line[x - 1];
                    match (
                        height.cmp(&last_height),
                        height.cmp(&y),
                        last_height.cmp(&y),
                    ) {
                        // already rendered above the graph
                        (_, Ordering::Greater, Ordering::Greater) => {}
                        // render below the graph
                        (_, Ordering::Less, Ordering::Less) => *span = Span::new("."),
                        // render the step between bins
                        (_, Ordering::Greater, Ordering::Less)
                        | (_, Ordering::Less, Ordering::Greater) => *span = Span::new(LINE_VERT),
                        // render the corner between bins
                        (Ordering::Equal, _, _) => *span = Span::new(LINE_HORIZONTAL),
                        (Ordering::Greater, Ordering::Equal, _) => {
                            *span = Span::new(CORNER_BTM_LEFT)
                        }
                        (Ordering::Less, Ordering::Equal, _) => *span = Span::new(CORNER_TOP_LEFT),
                        (Ordering::Greater, _, Ordering::Equal) => {
                            *span = Span::new(CORNER_TOP_RIGHT)
                        }
                        (Ordering::Less, _, Ordering::Equal) => *span = Span::new(CORNER_BTM_RIGHT),
                    }
                    span.style().fg(color);
                }
            }

            last_height = Some(height);
        }

        let mut lines = Vec::new();
        for ((legend, bin), boundary) in legend_bin.into_iter().zip(bins).zip(boundaries) {
            let mut line: Vec<Span> = vec![legend];
            line.extend(bin.into_iter().interleave(boundary));
            line.push(Span::new(format!("{:width$}", "", width = right_margin)));
            lines.push(line);
        }
        lines
    }

    /// Generate the legend_column and its width
    /// Assumes 0 is the top of the graph
    fn legend_column(&self) -> (Vec<Span>, usize) {
        let unit_str = format!("{}", self.midnight.swell.unit);
        let legend_max_str = format!("{}", self.max_swell_height);
        let legend_min_str = format!("{}", self.min_swell_height);
        let legend_num_str_len = legend_min_str.len().max(legend_max_str.len());
        let legend_width = legend_num_str_len + unit_str.len() + 2;
        let legend_max = format!(
            "{:>width$} {} ",
            legend_max_str,
            unit_str,
            width = legend_num_str_len
        );
        let legend_min = format!(
            "{:>width$} {} ",
            legend_min_str,
            unit_str,
            width = legend_num_str_len
        );
        let mut legend_bin = vec![
            Span::new(format!("{:width$}", "", width = legend_width));
            Self::SWELL_GRAPH_HEIGHT
        ];
        legend_bin[0] = Span::new(legend_max);
        legend_bin[Self::SWELL_GRAPH_HEIGHT - 1] = Span::new(legend_min);
        (legend_bin, legend_width)
    }

    fn scale(&self, height: f32) -> usize {
        let swell_range = (self.max_swell_height - self.min_swell_height) as f32;
        let proportion_of_range = (height - self.min_swell_height as f32) / swell_range;
        let scaled_to_graph = proportion_of_range * Self::SWELL_GRAPH_HEIGHT as f32;
        scaled_to_graph.round() as usize
    }

    /// The logic for coloring is actually quite limited; we don't have spot
    /// data for directions to determine whether or not swell/wind is
    /// on/off/cross-shore. Just using star rating as a proxy.
    fn color(fc: &Forecast) -> Color {
        match (fc.solid_rating, fc.faded_rating) {
            (0, _) => Color::Red,
            (_, 0) => Color::Green,
            (_, _) => Color::Blue,
        }
    }
}

pub struct Day<'a> {
    forecast: &'a [Forecast],
}

impl<'a> Day<'a> {
    pub fn new(forecast: &'a [Forecast]) -> Self {
        Self { forecast }
    }
    pub fn draw(self) -> Vec<Span> {
        vec![]
    }
}

/// Internal type synonym to distinguish line breaks on inner widgets
type Line = Vec<Span>;

/// A contiguous piece of content with consistent styles. These shouldn't need to
/// nest.
#[derive(Clone, Debug, PartialEq)]
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

/// Content is typically just text in the form of a String. But I think it will
/// make life easier to separate control chars like newlines. So, try not to
/// sneak those into the text values.
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Text(String),
    Newline,
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
