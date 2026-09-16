// Focused owners must expose inherent render rather than free builders.
pub struct BRow;
impl BRow {
    pub fn build(&self, c: &Ctx) -> Element { row(c) }
}
