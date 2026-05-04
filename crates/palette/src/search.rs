use crate::command::Command;
use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Matcher, Utf32Str};

/// Retourne les commandes filtrées par le query, triées par section puis par score.
/// C'est la SEULE source de vérité pour l'ordre des résultats.
pub fn filter_and_sort<'a>(commands: &'a [Command], query: &str) -> Vec<&'a Command> {
    if query.is_empty() {
        let mut all: Vec<&Command> = commands.iter().collect();
        all.sort_by_key(|c| c.section.order());
        return all;
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut buf = Vec::new();

    let mut scored: Vec<(u32, &Command)> = commands
        .iter()
        .filter_map(|cmd| {
            let haystack = Utf32Str::new(&cmd.name, &mut buf);
            pattern.score(haystack, &mut matcher).map(|s| (s, cmd))
        })
        .collect();

    // Tri principal : par section. Tri secondaire : par score décroissant.
    scored.sort_by_key(|(score, cmd)| (cmd.section.order(), std::cmp::Reverse(*score)));

    scored.into_iter().map(|(_, cmd)| cmd).collect()
}

/// Vérifie simplement si une commande matche (sans calculer le score).
/// Utile pour le compteur "X résultats" si on n'a pas besoin de l'ordre.
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