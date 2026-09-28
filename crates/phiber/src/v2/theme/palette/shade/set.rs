//! The reference table of the sphere's shades, as it was handed over —
//! the hex of each shade and how it is made — for the guard that keeps the
//! ladder honest and for the Preferences theme page that shows it.

/// Shade 1 … 18 of `#3B4657`: `(hex, how it is made)`.
pub const SPHERE_TABLE: [(&str, &str); 18] = [
    ("#060709", "base 10% over #000"),
    ("#0C0E11", "base 20% over #000"),
    ("#12151A", "base 30% over #000"),
    ("#181C23", "base 40% over #000"),
    ("#1E232C", "base 50% over #000"),
    ("#232A34", "base 60% over #000"),
    ("#29313D", "base 70% over #000"),
    ("#2F3846", "base 80% over #000"),
    ("#353F4E", "base 90% over #000"),
    ("#3B4657", "base 100% (the sphere)"),
    ("#4F5868", "white 10% over base"),
    ("#626B79", "white 20% over base"),
    ("#767E89", "white 30% over base"),
    ("#89909A", "white 40% over base"),
    ("#9DA2AB", "white 50% over base"),
    ("#B1B5BC", "white 60% over base"),
    ("#C4C8CD", "white 70% over base"),
    ("#D8DADD", "white 80% over base"),
];

/// How shade `step` of any ladder is made, in the table's words.
pub fn describe(step: u8) -> &'static str {
    let index = usize::from(step.clamp(1, super::STEPS)) - 1;
    SPHERE_TABLE[index].1
}
