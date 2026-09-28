//! Numbers typed while a transform runs — Blender's numeric input
//! (`numinput.cc`): digits and a point build the value, minus flips its
//! sign, Backspace takes the last character back; while there is a value
//! it, not the pointer, drives the transform.

/// The text typed so far.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BNumInput(pub String);

impl BNumInput {
    /// The text after `key`, or `None` when the key is not numeric input.
    pub fn with(&self, key: &str) -> Option<Self> {
        let mut text = self.0.clone();
        match key {
            "backspace" => {
                text.pop();
            }
            "-" | "minus" => match text.strip_prefix('-') {
                Some(rest) => text = rest.to_string(),
                None => text.insert(0, '-'),
            },
            // A second point is swallowed, not handed on to the keymap.
            "." | "period" => {
                if !text.contains('.') {
                    text.push('.');
                }
            }
            digit if digit.len() == 1 && digit.chars().all(|c| c.is_ascii_digit()) => text.push_str(digit),
            _ => return None,
        }
        Some(Self(text))
    }

    /// The value typed, once it is a number.
    pub fn value(&self) -> Option<f32> {
        self.0.parse().ok()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_a_point_and_minus_build_the_number() {
        let mut typed = BNumInput::default();
        for key in ["2", ".", "5"] {
            typed = typed.with(key).unwrap();
        }
        assert_eq!(typed.value(), Some(2.5));
        typed = typed.with("-").unwrap();
        assert_eq!(typed.value(), Some(-2.5), "minus flips it");
        typed = typed.with("backspace").unwrap();
        assert_eq!(typed.0, "-2.");
        assert_eq!(typed.with(".").unwrap().0, "-2.", "one point only");
        assert!(typed.with("x").is_none(), "not numeric input");
        assert_eq!(BNumInput("-".into()).value(), None);
    }
}
