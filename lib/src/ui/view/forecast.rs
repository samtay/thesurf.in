use std::{cmp::Ordering, fmt::Display};

use chrono::Timelike;
use itertools::Itertools;

use super::base::*;
use crate::msw::forecast::{
    CompassDirection, Forecast, SwellComponent, SwellComponents, UnitLength,
};

/// Total width of the viewpoint output. If the user's viewpoint is smaller than
/// this, output will look choppy, so we want to minimize it while keeping the
/// view useful.
const VIEWPOINT_WIDTH: usize = 90;
/// Viewpoint width minus the border chars
const INTERIOR_VIEWPOINT_WIDTH: usize = VIEWPOINT_WIDTH - 2;

impl From<Vec<Forecast>> for View {
    /// Transform a forecast into stylized text snippets
    fn from(forecast: Vec<Forecast>) -> Self {
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
        vec![span!(
            "{CORNER_BTM_LEFT}{:─^width$}{CORNER_BTM_RIGHT}",
            "",
            width = INTERIOR_VIEWPOINT_WIDTH
        )]
    }
}

/// The swell graph over a multi-day forecast
// TODO: gray out 6pm - 6am and add 6hr-x-axis ticks
struct Graph<'a> {
    forecast: &'a [Forecast],
    min_swell_height: f32,
    max_swell_height: f32,
    midnight: &'a Forecast,
}

impl Border for Graph<'_> {
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
            vec![span!("{:width$}", "", width = bin_width); num_bins];
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
                    Ordering::Equal => *span = span!("{:─^width$}", "", width = bin_width),
                    Ordering::Less => *span = span!("{:.^width$}", "", width = bin_width),
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
            line.push(span!("{:width$}", "", width = right_margin));
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
        let buffer = match forecast.first().unwrap().swell.unit {
            UnitLength::Feet => 1.0,
            UnitLength::Meters => 0.5,
        };
        let min_swell_height = 0.0;
        let max_swell_height = forecast
            .iter()
            .map(|fc| fc.swell.max_breaking_height)
            .fold(f32::NEG_INFINITY, |a, b| a.max(b))
            + buffer;
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
        let mut legend_bin =
            vec![span!("{:width$}", "", width = legend_width); Self::SWELL_GRAPH_HEIGHT];
        legend_bin[0] = Span::new(legend_max);
        legend_bin[Self::SWELL_GRAPH_HEIGHT - 1] = Span::new(legend_min);
        (legend_bin, legend_width)
    }

    fn scale(&self, height: f32) -> usize {
        let swell_range = self.max_swell_height - self.min_swell_height;
        let proportion_of_range = (height - self.min_swell_height) / swell_range;
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

    // Rows:
    //   Time
    //   Swell (primary, secondary if present)
    //     Height, Direction, Arrow, Period
    //   Wind
    //     Arrow, Dir, Speed
    //   Weather
    //     Air temp
    // Columns: 3hr intervals
    fn draw_inner(&self) -> Vec<Line> {
        let skip_line = vec![span!("{:^width$}", "", width = INTERIOR_VIEWPOINT_WIDTH)];

        let mut lines = vec![];
        lines.extend(self.time());
        lines.push(skip_line.clone());
        lines.extend(self.primary_swell());
        lines.push(skip_line.clone());
        if self.is_secondary_present() {
            lines.extend(self.secondary_swell());
            lines.push(skip_line.clone());
        }
        lines.extend(self.wind());
        lines.push(skip_line);
        lines.extend(self.weather());
        lines
    }
}

impl<'a> Day<'a> {
    // Primary / Secondary / Wind / Weather
    const LEGEND_WIDTH: usize = 11;
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

        Self {
            forecast,
            bin_width,
            right_margin,
        }
    }

    fn time(&self) -> Vec<Line> {
        // u23F2
        let mut time = Vec::with_capacity(2 * self.forecast.len() + 2);

        // Render the legend ( \u{23F2} for clock )
        time.push(span!("{:^width$}", "Time", width = Self::LEGEND_WIDTH));

        // Render each timestamp forecast
        for fc in self.forecast {
            time.push(self.boundary());
            time.push({
                span!(
                    "{:^width$}",
                    fc.local_timestamp.format("%l%P").to_string().trim(),
                    width = self.bin_width
                )
            });
        }
        time.push(span!("{:width$}", "", width = self.right_margin));

        vec![time]
    }

    fn primary_swell(&self) -> Vec<Line> {
        if self.is_primary_present() {
            self.swell("Primary", |sw: SwellComponents| sw.primary)
        } else {
            vec![]
        }
    }

    fn secondary_swell(&self) -> Vec<Line> {
        if self.is_secondary_present() {
            self.swell("Secondary", |sw: SwellComponents| sw.secondary)
        } else {
            vec![]
        }
    }

    fn swell<S, F>(&self, legend: S, component: F) -> Vec<Line>
    where
        S: Display,
        F: Fn(SwellComponents) -> Option<SwellComponent>,
    {
        const HEIGHT_IX: usize = 0;
        const PERIOD_IX: usize = 1;
        const DIR_IX: usize = 2;
        let bin_width = self.bin_width;
        let init = Vec::with_capacity(2 * self.forecast.len() + 2);
        let mut swell = [init.clone(), init.clone(), init];
        let empty = span!("{:^width$}", "", width = self.bin_width);

        // Render the legend // ↜, ↝
        swell[HEIGHT_IX].push(span!("{:^width$}", "", width = Self::LEGEND_WIDTH));
        swell[PERIOD_IX].push(span!("{:^width$}", legend, width = Self::LEGEND_WIDTH));
        swell[DIR_IX].push(span!("{:^width$}", "Swell", width = Self::LEGEND_WIDTH));

        // Render each timestamp forecast
        for fc in self.forecast {
            swell.iter_mut().for_each(|row| row.push(self.boundary()));
            //"{arrow} {deg:.0}° {height:.1} {unit} @ {period}s\n",
            if let Some(c) = component(fc.swell.components) {
                swell[HEIGHT_IX].push({
                    let str = format!(
                        "{height:.1} {unit}",
                        height = c.height,
                        unit = fc.swell.unit
                    );
                    span!("{:^bin_width$}", str)
                });
                swell[PERIOD_IX].push({
                    let str = format!("{period}s", period = c.period,);
                    span!("{:^bin_width$}", str)
                });
                swell[DIR_IX].push({
                    let str = format!(
                        "{arrow} {deg:.0}°",
                        arrow = compass_to_arrow(c.compass_direction),
                        deg = c.direction
                    );
                    span!("{:^bin_width$}", str)
                });
            } else {
                swell.iter_mut().for_each(|row| row.push(empty.clone()));
            }
        }
        swell
            .iter_mut()
            .for_each(|row| row.push(span!("{:width$}", "", width = self.right_margin)));

        swell.to_vec()
    }

    fn wind(&self) -> Vec<Line> {
        const SPEED_IX: usize = 0;
        const DIR_IX: usize = 1;
        let init = Vec::with_capacity(2 * self.forecast.len() + 2);
        let mut wind = [init.clone(), init];

        // Render the legend // use 🌫
        wind[SPEED_IX].push(span!("{:^width$}", " ", width = Self::LEGEND_WIDTH));
        wind[DIR_IX].push(span!("{:^width$}", " Wind", width = Self::LEGEND_WIDTH));

        // Render each timestamp forecast
        for fc in self.forecast {
            wind.iter_mut().for_each(|row| row.push(self.boundary()));
            wind[SPEED_IX].push({
                let str = format!("{speed} {unit}", speed = fc.wind.speed, unit = fc.wind.unit);
                span!("{:^width$}", str, width = self.bin_width)
            });
            wind[DIR_IX].push({
                let str = format!(
                    "{arrow} {deg:.0}°",
                    arrow = compass_to_arrow(fc.wind.compass_direction),
                    deg = fc.wind.direction
                );
                span!("{:^width$}", str, width = self.bin_width)
            });
        }
        wind.iter_mut()
            .for_each(|row| row.push(span!("{:width$}", "", width = self.right_margin)));

        wind.to_vec()
    }

    fn weather(&self) -> Vec<Line> {
        // ☼ 🌣 🌤 🌧 🌩
        let mut weather = Vec::with_capacity(2 * self.forecast.len() + 2);

        // Render the legend // unicode icons: ☼ 🌣 🌤 🌧 🌩
        weather.push(span!("{:^width$}", "Air", width = Self::LEGEND_WIDTH));

        // Render each timestamp forecast
        for fc in self.forecast {
            weather.push(self.boundary());
            weather.push({
                let str = format!(
                    "{temp} {unit}",
                    temp = fc.condition.temperature,
                    unit = fc.condition.unit_temperature
                );
                span!("{:^width$}", str, width = self.bin_width)
            });
        }
        weather.push(span!("{:width$}", "", width = self.right_margin));

        vec![weather]
    }

    /// The span between columns
    fn boundary(&self) -> Span {
        //span!("{LINE_VERT}")
        span!("{:^width$}", "", width = Self::BOUNDARY_WIDTH)
    }

    /// Check if any primary swell in the forecast
    fn is_primary_present(&self) -> bool {
        self.forecast
            .iter()
            .map(|fc| fc.swell.components.primary)
            .any(|c| c.is_some())
    }

    /// Check if any secondary swell in the forecast
    fn is_secondary_present(&self) -> bool {
        self.forecast
            .iter()
            .map(|fc| fc.swell.components.secondary)
            .any(|c| c.is_some())
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
