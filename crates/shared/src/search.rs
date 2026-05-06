use crate::command::{Command, Section};
use crate::mode::Mode;
use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Matcher, Utf32Str};

pub const MAX_RESULTS: usize = 8;

pub fn filter_and_sort(commands: &[Command], mode: Mode, query: &str) -> Vec<Command> {
    let mode_filtered: Vec<&Command> = commands
        .iter()
        .filter(|cmd| match mode {
            Mode::Universal => !matches!(
                cmd.section,
                Section::System | Section::Settings
            ),
            Mode::Commands => matches!(
                cmd.section,
                Section::System | Section::Settings
            ),
            _ => false,
        })
        .collect();

    if query.is_empty() {
        let mut all = mode_filtered;
        all.sort_by_key(|c| c.section.order());
        return all.into_iter().take(MAX_RESULTS).cloned().collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut buf = Vec::new();

    let mut scored: Vec<(u32, &Command)> = mode_filtered
        .into_iter()
        .filter_map(|cmd| {
            let haystack = Utf32Str::new(&cmd.name, &mut buf);
            pattern.score(haystack, &mut matcher).map(|s| (s, cmd))
        })
        .collect();

    scored.sort_by_key(|(score, cmd)| (cmd.section.order(), std::cmp::Reverse(*score)));
    scored.into_iter().take(MAX_RESULTS).map(|(_, cmd)| cmd.clone()).collect()
}
