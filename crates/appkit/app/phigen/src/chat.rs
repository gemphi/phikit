//! A chat: what was said, in order, and by whom.

/// Who said something.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Who {
    Person,
    Phigen,
}

/// One thing said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub who: Who,
    pub text: String,
}

impl Turn {
    pub fn new(who: Who, text: &str) -> Turn {
        Turn { who, text: text.to_string() }
    }
}

/// What was said, in order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Chat {
    pub turn: Vec<Turn>,
    /// When the person first said something in it, in seconds since
    /// 1970; nothing while that is not known.
    pub when: u64,
}

impl Chat {
    /// A chat Phigen opens with a greeting.
    pub fn greeted(greeting: &str) -> Chat {
        Chat { turn: vec![Turn::new(Who::Phigen, greeting)], when: 0 }
    }

    /// What the chat is called: the first thing the person said. `None`
    /// until they have said something.
    pub fn title(&self) -> Option<&str> {
        self.turn.iter().find(|turn| turn.who == Who::Person).map(|turn| turn.text.as_str())
    }
}
