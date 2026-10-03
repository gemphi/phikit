#[allow(unused_imports)]
use super::*;

/// The prompt a quest asks, and the shape the answer must come back in.
///
/// The format is the whole trick. A model asked for prose returns prose, and
/// prose has to be parsed by another model; a model asked for one relation per
/// line, pipe-separated, returns that, and it parses with `split('|')`. The
/// template is data so it can be edited without touching the loop, and so a
/// quest can record which template produced which edge.
#[derive(Clone, Debug)]
pub struct Template {
    pub name: &'static str,
    pub body: &'static str,
}

/// Ask what a word relates to. `{word}` and `{kinds}` are filled in.
pub const RELATIONS: Template = Template {
    name: "relations",
    body: "\
You are describing one word for a relation engine, not for a reader.
A word means nothing except what it relates to, under which condition.

Word: {word}

List at most 8 relations this word stands in. One per line, exactly:

kind | other word | weight

- kind: one of: {kinds}
- other word: a single common word, lower case, no phrase
- weight: 0.1 to 1.0, how strongly the relation holds
- the other word is never the word itself, and never the kind

For example, for a different word, bread:

is-a | food | 0.9
made-of | flour | 0.9
used-for | eating | 0.8

No prose, no numbering, no blank lines, nothing after the last relation.",
};

/// Ask one kind about one word. `{kinds}` is the one kind.
pub const ONE_KIND: Template = Template {
    name: "one-kind",
    body: "\
You are describing one word for a relation engine, not for a reader.

Word: {word}
Relation: {kinds}

List at most 3 words that {word} stands in this relation to. One per line, exactly:

{kinds} | other word | weight

- other word: a single common word, lower case, no phrase, never {word} itself
- weight: 0.1 to 1.0, how strongly the relation holds

No prose, no numbering, no blank lines, nothing after the last line.",
};

/// Ask for a persona's own sentences with a word. `{kinds}` is the voice.
pub const VOICE: Template = Template {
    name: "voice",
    body: "\
Word: {word}

Write 2 short sentences that use the word {word}, {kinds}.

One sentence per line. No numbering, no quotes, nothing before or after.",
};

/// Ask which conditions change what a word calls for.
pub const CONDITIONS: Template = Template {
    name: "conditions",
    body: "\
Word: {word}

Name at most 4 conditions under which this word relates to different things,
and what it relates to under each. One per line, exactly:

condition | other word | weight

- condition: a single common word naming the setting or sense
- other word: a single common word, lower case
- weight: 0.1 to 1.0

No prose, no numbering.",
};

impl Template {
    /// Fill `{word}` and `{kinds}`.
    pub fn fill(&self, word: &str, kinds: &[&str]) -> String {
        self.body.replace("{word}", word).replace("{kinds}", &kinds.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parser_survives_a_chatty_model() {
        let answer = "\
Sure! Here are the relations for `cat`:

```
is-a | animal | 0.9
- has | fur | 0.8
does  |  sleep  | 0.6
this line is prose and should be dropped
part-of | a long phrase here | 0.5
near | dog |
```
Hope this helps!";
        let claims = parse_relations(answer, &[]);
        assert_eq!(claims.len(), 4, "prose and phrases dropped, bullets kept");
        assert_eq!(claims[0], Claim { kind: "is-a".into(), other: "animal".into(), weight: 0.9 });
        assert_eq!(claims[1].kind, "has");
        assert_eq!(claims[2].other, "sleep");
        assert_eq!(claims[3].weight, 1.0, "a missing weight is a full one");
    }

    #[test]
    fn filling_a_template_names_the_kinds() {
        let p = RELATIONS.fill("cat", &["is-a", "has"]);
        assert!(p.contains("Word: cat"));
        assert!(p.contains("one of: is-a, has"));
    }
}

