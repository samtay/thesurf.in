use std::cmp::Ordering;

use chrono::Timelike;
use itertools::Itertools;

use crate::msw::forecast::{CompassDirection, Forecast};

/// Total width of the viewpoint output. If the user's viewpoint is smaller than
/// this, output will look choppy, so we want to minimize it while keeping the
/// view useful.
const VIEWPOINT_WIDTH: usize = 90;
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
            if !fc.is_empty() {
                spans.push(Span::newline());
                spans.extend(Day::new(fc).draw());
            }
        }
        Self { spans }
    }
}

/// TODO maybe move base stuff to internal mod
// Also: make a more elegant abstraction than this?
trait Border {
    /// Title string (unpadded)
    fn title(&self) -> String;

    /// Contents within the border
    fn draw_inner(&self) -> Vec<Line>;

    /// Default `draw()` is to border the contents within the border.
    fn draw(self) -> Vec<Span>
    where
        Self: Sized,
    {
        self.border(self.draw_inner())
    }

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
// TODO: gray out 6pm - 6am and add 6hr-x-axis ticks
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

    fn draw_inner(&self) -> Vec<Line> {
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
}

impl<'a> Graph<'a> {
    const SWELL_GRAPH_HEIGHT: usize = 10;

    /// Panics on empty forecast
    pub fn new(forecast: &'a [Forecast]) -> Self {
        assert!(!forecast.is_empty());
        // TODO some smartness for a good graph range.
        let min_swell_height = 0;
        let max_swell_height =
            // 10.max(
            forecast
                .iter()
                .map(|fc| fc.swell.max_breaking_height)
                .max()
                .unwrap_or(5)
                + 1;
        let midnight = forecast.first().unwrap();
        Self {
            forecast,
            min_swell_height,
            max_swell_height,
            midnight,
        }
    }

    /// Generate the legend_column and its width
    /// Assumes 0 is the top of the graph
    fn legend_column(&self) -> (Vec<Span>, usize) {
        let unit_str = format!("{}", self.midnight.swell.unit);
        let legend_max_str = format!("{}", self.max_swell_height);
        let legend_min_str = format!("{}", self.min_swell_height);
        let legend_num_str_len = legend_min_str.len().max(legend_max_str.len());
        let legend_max = format!(
            " {:>width$} {} ",
            legend_max_str,
            unit_str,
            width = legend_num_str_len
        );
        let legend_min = format!(
            " {:>width$} {} ",
            legend_min_str,
            unit_str,
            width = legend_num_str_len
        );
        assert_eq!(legend_max.len(), legend_min.len());
        let legend_width = legend_max.len();
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
    bin_width: usize,
    right_margin: usize,
}

impl Border for Day<'_> {
    /// Gen title for the week's swell graph
    fn title(&self) -> String {
        let date = self
            .forecast
            .iter()
            .next()
            .unwrap()
            .local_timestamp
            .format("%a %b %d");
        format!("{date}")
    }

    fn draw_inner(&self) -> Vec<Line> {
        // Rows:
        //   Swell (primary, secondary if present)
        //     Height, Direction, Arrow, Period
        //   Wind
        //     Arrow, Dir, Speed
        //   Weather
        //     Air temp
        // Columns: 3hr intervals

        // TODO no, with strict viewport we just need to force width
        // TODO probably want to generate all strings first, and then choose a a
        // minimum common width to keep all columns consistent?
        // TODO also need to set max width for all cols based on viewport

        // Note that we assume each span represents a single (row,col) cell,
        // and will inspect the text to find the common width.
        let swell_lines = self.swell();
        let wind_lines = self.wind();
        let weather_lines = self.weather();

        swell_lines
    }
}

impl<'a> Day<'a> {
    // Primary / Secondary / Wind / Weather
    const LEGEND_WIDTH: usize = 9;
    const BOUNDARY_WIDTH: usize = 1;

    /// Panics on empty forecast
    pub fn new(forecast: &'a [Forecast]) -> Self {
        assert!(!forecast.is_empty());

        // should be 8
        let num_forecasts = forecast.len();
        // between each forecast, and between legend and first forecast
        let num_boundaries = num_forecasts;
        let bin_width =
            (INTERIOR_VIEWPOINT_WIDTH - num_boundaries * Self::BOUNDARY_WIDTH - Self::LEGEND_WIDTH)
                / num_forecasts;
        let used_space =
            Self::LEGEND_WIDTH + num_boundaries * Self::BOUNDARY_WIDTH + num_forecasts * bin_width;
        let right_margin = INTERIOR_VIEWPOINT_WIDTH - used_space;
        dbg!(
            &num_forecasts,
            &bin_width,
            &used_space,
            &right_margin,
            INTERIOR_VIEWPOINT_WIDTH,
            VIEWPOINT_WIDTH
        );

        Self {
            forecast,
            bin_width,
            right_margin,
        }
    }

    // TODO both optional! skip lines when not present? or 0ft?
    // TODO legend column with Primary / Secondary text
    fn swell(&self) -> Vec<Line> {
        // Skip rendering if there's no swell?
        if self
            .forecast
            .iter()
            .map(|fc| fc.swell.components.primary)
            .all(|c| c.is_none())
        {
            return vec![];
        }

        // 12am, 3am, 6am, 9am, 12pm, 3pm, 6pm, 9am
        let len = self.forecast.len();
        let mut primary_height = Vec::with_capacity(len);
        let mut primary_period = Vec::with_capacity(len);
        let mut primary_direction = Vec::with_capacity(len);
        let empty = Span::new(format!("{:^width$}", "", width = self.bin_width));
        let boundary = Span::new(format!("{:^width$}", "", width = Self::BOUNDARY_WIDTH));

        // Render the legend
        primary_height.push(Span::new(format!(
            "{:^width$}",
            "↜",
            width = Self::LEGEND_WIDTH
        )));
        primary_period.push(Span::new(format!(
            "{:^width$}",
            "Primary",
            width = Self::LEGEND_WIDTH
        )));
        primary_direction.push(Span::new(format!(
            "{:^width$}",
            "↝",
            width = Self::LEGEND_WIDTH
        )));

        // Render each timestamp forecast
        for fc in self.forecast {
            primary_height.push(boundary.clone());
            primary_period.push(boundary.clone());
            primary_direction.push(boundary.clone());
            //"{arrow} {deg:.0}° {height:.1} {unit} @ {period}s\n",
            let component = fc.swell.components.primary;
            primary_height.push(
                component
                    .map(|c| {
                        let str = format!(
                            "{height:.1} {unit}",
                            height = c.height,
                            unit = fc.swell.unit
                        );
                        Span::new(format!("{:^width$}", str, width = self.bin_width))
                    })
                    .unwrap_or_else(|| empty.clone()),
            );
            primary_period.push(
                component
                    .map(|c| {
                        let str = format!("{period}s", period = c.period,);
                        Span::new(format!("{:^width$}", str, width = self.bin_width))
                    })
                    .unwrap_or_else(|| empty.clone()),
            );
            primary_direction.push(
                component
                    .map(|c| {
                        let str = format!(
                            "{arrow} {deg:.0}°",
                            arrow = compass_to_arrow(c.compass_direction),
                            deg = c.direction
                        );
                        Span::new(format!("{:^width$}", str, width = self.bin_width))
                    })
                    .unwrap_or_else(|| empty.clone()),
            );
        }
        primary_height.push(Span::new(format!(
            "{:width$}",
            "",
            width = self.right_margin
        )));
        primary_period.push(Span::new(format!(
            "{:width$}",
            "",
            width = self.right_margin
        )));
        primary_direction.push(Span::new(format!(
            "{:width$}",
            "",
            width = self.right_margin
        )));

        vec![primary_height, primary_period, primary_direction]

        // TODOhandle secondary later?
    }

    fn wind(&self) -> Vec<Line> {
        // 🌫
        vec![]
    }

    fn weather(&self) -> Vec<Line> {
        // ☼ 🌣 🌤 🌧 🌩
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

fn compass_to_arrow(dir: CompassDirection) -> &'static str {
    use CompassDirection::*;
    match dir {
        N => "↓",
        NNE | NE | ENE => "↙",
        E => "←",
        ESE | SE | SSE => "↖",
        S => "↑",
        SSW | SW | WSW => "↗",
        W => "→",
        WNW | NW | NNW => "↘",
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
