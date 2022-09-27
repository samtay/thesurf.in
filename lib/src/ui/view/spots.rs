use super::base::*;

/// Looks like the current max is 9278, an extra order of magnitude should be good.
const SPOT_ID_LEN_MAX: usize = 5;

impl From<Vec<(String, u16)>> for View {
    /// Transform a forecast into stylized text snippets
    fn from(spots: Vec<(String, u16)>) -> Self {
        let max_str = spots
            .iter()
            .max_by_key(|t| t.0.len())
            .map(|t| t.0.len())
            .unwrap_or(20);
        let mut spots = spots;
        spots.sort_unstable_by(|t, s| t.0.cmp(&s.0));
        let mut spans = Vec::with_capacity(spots.len() * 2);
        for (name, id) in spots {
            spans.push(span!(
                "{:>n_width$} : {:<id_width$}",
                name,
                id,
                n_width = max_str,
                id_width = SPOT_ID_LEN_MAX
            ));
            spans.push(Span::newline());
        }
        Self { spans }
    }
}
