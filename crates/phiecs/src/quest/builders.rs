//! The builders' game as a quest (Philosophical Investigations §2, §8, §21,
//! §27): the oracle is put in the game and asked to *act*, not to relate.
//! Its reply — one action in five aspects — is what a student learns the
//! language from, the way the assistant in §2 learns it: by what follows a
//! call. The prompt states the game's set-up (the stones, the letters used
//! as numerals, the places); it never says what any call means.

/// The five aspects of what follows a call, in reply order.
pub const ASPECTS: [&str; 5] = ["act", "what", "colour", "count", "place"];

/// The question for one episode. `game` is `order` (A calls, B acts) or
/// `report` (B tells A; A records); `gestures` are the pointing that goes
/// with the words, e.g. `pointing at place p3`.
pub fn prompt(game: &str, words: &str, gestures: &[String]) -> String {
    prompt_with(game, words, gestures, false)
}

/// The same question; with `roles`, the scene also says what each player
/// does in the game — who fetches and who only takes note — without saying
/// what any call means (the second Phi-4 run, docs/107: the first mixed up
/// the two games' acts).
pub fn prompt_with(game: &str, words: &str, gestures: &[String], roles: bool) -> String {
    prompt_shown(game, words, gestures, roles, &[])
}

/// The same question after A has taught a new word by pointing (§28–30):
/// each of `shown` is one such teaching, told as what was pointed at and
/// what was said (docs/116). The reply form then allows a taught word.
pub fn prompt_shown(game: &str, words: &str, gestures: &[String], roles: bool, shown: &[String]) -> String {
    let before = shown.iter().map(|x| format!("Earlier, {x}\n")).collect::<String>();
    let taught = if shown.is_empty() { "" } else { ", or a word A taught you by pointing" };
    let scene = match (game, roles) {
        ("report", false) => "B, the assistant, reports to A, the builder. You are A: say what you do on hearing it.",
        ("report", true) => "B, the assistant, reports to A, the builder, what is there. You are A: you only take note of what you hear; you do not fetch anything. Say what you do on hearing it.",
        (_, false) => "A, the builder, calls out to you. You are B, the assistant: say what you do.",
        (_, true) => "A, the builder, calls out to you. You are B, the assistant: A tells you what to do and you do it; you do not take notes. Say what you do.",
    };
    let pointing = if gestures.is_empty() { String::new() } else { format!(" ({})", gestures.join(", ")) };
    format!(
        "A language game of builders (Wittgenstein, Philosophical Investigations §2 and §8). The stones are block, pillar, slab and beam. \
The letters a to j are used as numerals (a = 1, b = 2, … j = 10). The colours are red, blue and yellow. The places are here and p1 to p5.\n\
{scene}\n\
{before}Said: \"{words}\"{pointing}\n\
Answer with exactly one line and nothing else, in this form:\n\
act | what | colour | count | place\n\
act is one of bring, come, clear, undo, record; what is a stone or -; colour is red, blue, yellow, any or -{taught}; count is a number from 1 to 10 or -; place is here, p1 … p5 or -."
    )
}

/// The first reply line with five `|`-separated fields, normalised: lower
/// case, numerals as digits (the letters a–j too, in the count, since the
/// prompt declares them numerals), `none` as `-`. Terminal escapes are
/// removed, and a markdown table rule (`--- | --- | …`) is not an answer.
/// `None` when no line fits.
pub fn parse(reply: &str) -> Option<[String; 5]> {
    let reply = strip_escapes(reply);
    for line in reply.lines() {
        let line = line.trim().trim_matches(|c| c == '`' || c == '*' || c == '"');
        let fields: Vec<String> = line.split('|').map(|f| f.trim().to_lowercase()).collect();
        if fields.len() != 5 || fields.iter().any(String::is_empty) || fields[0] == "act" || fields.iter().all(|f| f.chars().all(|c| c == '-' || c == ':')) && fields.iter().any(|f| f.len() > 1) {
            continue;
        }
        let mut out: [String; 5] = Default::default();
        for (slot, f) in out.iter_mut().zip(&fields) {
            *slot = match f.as_str() {
                "none" | "n/a" | "—" => "-".to_string(),
                x => word_number(x).unwrap_or(x).to_string(),
            };
        }
        if let Some(i) = LETTERS.iter().position(|l| *l == out[3]) {
            out[3] = (i + 1).to_string();
        }
        return Some(out);
    }
    None
}

const LETTERS: [&str; 10] = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

/// Renders the terminal escapes a TTY-bound oracle such as `ollama run`
/// writes while it streams: `ESC [ n D` moves the cursor back n places and
/// the text after it overwrites what was there, so those n characters are
/// dropped; every other sequence (`ESC [ K`, colours) is removed.
pub fn strip_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        if it.peek() != Some(&'[') {
            continue;
        }
        it.next();
        let mut n = String::new();
        while let Some(x) = it.next() {
            if x.is_ascii_alphabetic() {
                if x == 'D' {
                    for _ in 0..n.parse::<usize>().unwrap_or(1) {
                        if out.ends_with('\n') || out.pop().is_none() {
                            break;
                        }
                    }
                }
                break;
            }
            n.push(x);
        }
    }
    out
}

fn word_number(x: &str) -> Option<&'static str> {
    const WORDS: [&str; 10] = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];
    const DIGITS: [&str; 10] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];
    WORDS.iter().position(|w| *w == x).map(|i| DIGITS[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reply_is_read_as_five_aspects() {
        let r = "Sure.\n**bring | slab | red | Four | p2**\n";
        assert_eq!(parse(r).unwrap(), ["bring", "slab", "red", "4", "p2"].map(String::from));
        assert!(parse("act | what | colour | count | place").is_none(), "the form itself is not an answer");
        assert_eq!(parse("come | none | - | - | -").unwrap()[1], "-");
    }

    #[test]
    fn escapes_rules_and_letter_numerals_are_read() {
        let r = "--- | --- | --- | --- | ---\nbring | slab | any | g | p\u{1b}[2D\u{1b}[Kp2";
        assert_eq!(parse(r).unwrap(), ["bring", "slab", "any", "7", "p2"].map(String::from));
        assert_eq!(parse("clear | - | - | - | -").unwrap()[0], "clear", "a single dash is an answer");
    }

    #[test]
    fn the_prompt_sets_the_game_not_the_meanings() {
        let p = prompt("order", "d slab there", &["pointing at place p3".into()]);
        assert!(p.contains("\"d slab there\" (pointing at place p3)"));
        assert!(!p.contains("help means"));
        let r = prompt_with("report", "d slab", &[], true);
        assert!(r.contains("only take note") && !r.contains("act is record"));
    }
}
