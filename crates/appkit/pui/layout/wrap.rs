//! Breaking text into the lines that fit a width.

/// The lines `text` is set in when none may be wider than `width`, as
/// `gauge` measures it. A line breaks between words; a word wider than
/// the width keeps a line to itself. A line break in the text is kept.
pub fn wrap(text: &str, width: f32, gauge: &dyn Fn(&str) -> f32) -> Vec<String> {
    let mut out = Vec::new();
    for passage in text.lines() {
        let mut line = String::new();
        for word in passage.split_whitespace() {
            let tried = match line.is_empty() {
                true => word.to_string(),
                false => format!("{line} {word}"),
            };
            match line.is_empty() || gauge(&tried) <= width {
                true => line = tried,
                false => out.push(std::mem::replace(&mut line, word.to_string())),
            }
        }
        out.push(line);
    }
    out
}

/// The same, and no line wider than `width` at all: a word too long for
/// a line, such as the path of a folder, is cut where it stops fitting
/// and goes on in the next line.
pub fn fold(text: &str, width: f32, gauge: &dyn Fn(&str) -> f32) -> Vec<String> {
    let mut out = Vec::new();
    for line in wrap(text, width, gauge) {
        let mut piece = String::new();
        for letter in line.chars() {
            piece.push(letter);
            // A piece of one letter stays, however narrow the room.
            if gauge(&piece) > width && piece.chars().count() > 1 {
                piece.pop();
                out.push(std::mem::replace(&mut piece, letter.to_string()));
            }
        }
        out.push(piece);
    }
    out
}
