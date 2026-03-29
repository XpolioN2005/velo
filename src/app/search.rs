use crate::command::CommandRegistry;
use std::rc::Rc;

pub struct MatchedCommand {
    pub cmd: Rc<crate::command::Command>,
    pub name: String,
    pub description: String,
    pub match_indices: Vec<usize>,
    pub score: i32,
}

pub fn build_results(query: &str, commands: &CommandRegistry) -> Vec<MatchedCommand> {
    let mut results = Vec::new();

    if query.is_empty() {
        return results;
    }

    let q = query.to_lowercase();

    for cmd in commands.iter() {
        if let Some((score, indices)) = fuzzy_match(&cmd.name, &q) {
            results.push(MatchedCommand {
                cmd: Rc::clone(cmd),
                name: cmd.name.clone(),
                description: cmd.description.clone(),
                match_indices: indices,
                score,
            });
            continue;
        }

        // match against aliases
        if let Some(score) = cmd
            .aliases
            .iter()
            .filter_map(|a| fuzzy_match(a, &q).map(|(s, _)| s + 15))
            .max()
        {
            results.push(MatchedCommand {
                cmd: Rc::clone(cmd),
                name: cmd.name.clone(),
                description: cmd.description.clone(),
                match_indices: vec![],
                score,
            });
        }
    }

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

fn fuzzy_match(name: &str, query: &str) -> Option<(i32, Vec<usize>)> {
    let name_lower = name.to_lowercase();
    let name_chars: Vec<char> = name_lower.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();

    let mut indices = Vec::new();
    let mut score = 0i32;
    let mut ni = 0;
    let mut qi = 0;
    let mut last_match = None::<usize>;
    let mut consecutive = 0i32;

    while qi < query_chars.len() && ni < name_chars.len() {
        if name_chars[ni] == query_chars[qi] {
            indices.push(ni);

            if ni > 0 && last_match == Some(ni - 1) {
                consecutive += 1;
                score += 5 * consecutive;
            } else {
                consecutive = 0;
            }

            if ni == 0 {
                score += 10;
            }

            if ni > 0 && (name_chars[ni - 1] == ' ' || name_chars[ni - 1] == '_') {
                score += 8;
            }

            score += 1;
            last_match = Some(ni);
            qi += 1;
        }
        ni += 1;
    }

    if qi == query_chars.len() {
        Some((score, indices))
    } else {
        None
    }
}
