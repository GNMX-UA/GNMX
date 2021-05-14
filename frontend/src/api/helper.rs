pub fn make_suggestions(names: &[&str]) -> Suggestions {
    names
        .iter()
        .enumerate()
        .map(
            (|(i, s)| Suggestion {
                name: s.to_string(),
                value: i as i64,
            }),
        )
        .collect()
}