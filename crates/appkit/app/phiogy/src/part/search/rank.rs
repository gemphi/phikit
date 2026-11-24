//! How well something answers what is sought, as Phiber's palette finds
//! (phiber: modal/command/find): every word sought must begin a word of
//! its name or of its line, small letters and capitals alike. What is
//! found is ranked: a name that begins with what is sought first, then
//! a name with every word, then a line with them; nothing sought finds
//! everything, in its own order.

/// The words of a text, small, split where a letter or figure ends.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase().split(|one: char| !one.is_alphanumeric()).filter(|word| !word.is_empty()).map(str::to_string).collect()
}

/// Its rank, the lower the better; none where it does not answer.
pub fn rank(sought: &str, name: &str, line: &str) -> Option<u8> {
    let wanted = words(sought);
    if wanted.is_empty() {
        return Some(0);
    }
    let (own, said) = (words(name), words(line));
    let begun = |list: &[String]| wanted.iter().all(|want| list.iter().any(|word| word.starts_with(want.as_str())));
    let both: Vec<String> = own.iter().chain(&said).cloned().collect();
    match (name.to_lowercase().starts_with(&sought.trim().to_lowercase()), begun(&own), begun(&both)) {
        (true, ..) => Some(0),
        (_, true, _) => Some(1),
        (_, _, true) => Some(2),
        _ => None,
    }
}

/// Things ranked: those that answer, the best first, the rest in their
/// own order; no more than `most`.
pub fn sift<T>(list: impl IntoIterator<Item = T>, sought: &str, most: usize, said: impl Fn(&T) -> (String, String)) -> Vec<T> {
    let mut found: Vec<(u8, usize, T)> = list.into_iter().enumerate().filter_map(|(at, one)| {
        let (name, line) = said(&one);
        rank(sought, &name, &line).map(|rank| (rank, at, one))
    }).collect();
    found.sort_by_key(|(rank, at, _)| (*rank, *at));
    found.into_iter().take(most).map(|(.., one)| one).collect()
}

#[cfg(test)]
mod tests {
    use super::rank;

    #[test]
    fn every_word_sought_begins_a_word_and_a_name_that_begins_with_it_comes_first() {
        assert_eq!(rank("once", "Once City", "Urban"), Some(0));
        assert_eq!(rank("city once", "Once City", ""), Some(1));
        assert_eq!(rank("urb", "Once City", "Urban & City Zones"), Some(2));
        assert_eq!(rank("ty", "Once City", "Urban"), None);
        assert_eq!(rank("  ", "Anything", ""), Some(0));
    }
}
