use crate::command::{system_commands, Command};
use crate::mode::Mode;
use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Matcher, Utf32Str};

/// Retourne les commandes filtrées selon le mode et le query effectif.
pub fn filter_and_sort<'a>(commands: &'a [Command], mode: Mode, query: &str) -> Vec<&'a Command> {
    let pool: &[Command];

    // En mode commandes pures (>), on utilise system_commands à la place
    // On ne peut pas les stocker dans commands (lifetimes), donc on les filtre à part
    // via une version statique — voir note ci-dessous.
    // Pour l'instant on utilise le pool passé en paramètre avec un filtre de section.

    let mode_filtered: Vec<&Command> = commands
        .iter()
        .filter(|cmd| match mode {
            Mode::Universal => !matches!(
                cmd.section,
                crate::command::Section::System | crate::command::Section::Settings
            ),
            Mode::Commands => matches!(
                cmd.section,
                crate::command::Section::System | crate::command::Section::Settings
            ),
            // Les autres modes n'utilisent pas ce pool de commandes
            _ => false,
        })
        .collect();

    if query.is_empty() {
        let mut all = mode_filtered;
        all.sort_by_key(|c| c.section.order());
        return all;
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
    scored.into_iter().map(|(_, cmd)| cmd).collect()
}

/// Combine les commandes selon le mode pour construire le pool complet.
/// Appelé une fois à l'init depuis app.rs pour peupler self.commands.
pub fn build_command_pool() -> Vec<Command> {
    system_commands()
}

pub fn matches(name: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut buf = Vec::new();
    let haystack = Utf32Str::new(name, &mut buf);
    pattern.score(haystack, &mut matcher).is_some()
}
