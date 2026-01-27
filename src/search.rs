use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use std::collections::HashMap;
use std::sync::Arc;

use crate::parse::DicomInfo;

pub fn fuzzy_filter_series(
    groups: HashMap<Arc<str>, Vec<DicomInfo>>,
    pattern: &str,
) -> Vec<(Arc<str>, Vec<DicomInfo>, u32)> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(pattern, CaseMatching::Ignore, Normalization::Smart);

    let mut matches: Vec<(Arc<str>, Vec<DicomInfo>, u32)> = groups
        .into_iter()
        .filter_map(|(description, files)| {
            let mut buf = Vec::new();
            let haystack = nucleo_matcher::Utf32Str::new(&description, &mut buf);
            pattern
                .score(haystack, &mut matcher)
                .map(|score| (description, files, score))
        })
        .collect();

    // Sort by score descending (best matches first)
    matches.sort_by(|a, b| b.2.cmp(&a.2));

    matches
}
