//! `Pose` and `WorldMatrix` as stored, timed aspects.
use super::{Pose, WorldMatrix};
use crate::comp::stored::{Fields, Stored, Value, field};
impl Stored for Pose {
    const KIND: &'static str = "pose";
    const TIMED: bool = true;
    fn fields(&self) -> Fields {
        vec![("t".into(), Value::nums(&self.t)), ("r".into(), Value::nums(&self.r)), ("s".into(), Value::nums(&self.s))]
    }
    fn from_fields(f: &Fields) -> Option<Self> {
        Some(Self { t: field(f, "t")?.as_nums()?, r: field(f, "r")?.as_nums()?, s: field(f, "s")?.as_nums()? })
    }
}
impl Stored for WorldMatrix {
    const KIND: &'static str = "world";
    const TIMED: bool = true;
    /// Sixteen numbers, column by column.
    fn fields(&self) -> Fields {
        let flat: Vec<f32> = self.0.iter().flatten().copied().collect();
        vec![("m".into(), Value::nums(&flat))]
    }
    fn from_fields(f: &Fields) -> Option<Self> {
        let flat: [f32; 16] = field(f, "m")?.as_nums()?;
        Some(Self(std::array::from_fn(|j| std::array::from_fn(|i| flat[4 * j + i]))))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::aspect::Aspect;
    #[test]
    fn pose_and_matrix_round_trip() {
        let p = Pose { t: [1.0, 2.5, -3.0], r: Pose::turned([0.0, 1.0, 0.0], 0.3), s: [1.0, 2.0, 3.0] };
        assert_eq!(Pose::from_fields(&p.fields()), Some(p));
        let m = p.matrix();
        assert_eq!(WorldMatrix::from_fields(&m.fields()), Some(m));
        assert_eq!(Pose::from_fields(&vec![]), None);
        assert_eq!((Pose::name(), WorldMatrix::name()), (Pose::KIND, WorldMatrix::KIND));
    }
}
