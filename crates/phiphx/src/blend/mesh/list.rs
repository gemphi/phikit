//! Faces of one mesh, flat.
//!
//! A `Vec<Vec<u32>>` costs a heap allocation and ~56 bytes of book-keeping per
//! face; a 20 M-face file spends over a gigabyte on that alone. [`BFaces`]
//! keeps every corner in one array with a start per face, so the same file
//! costs 4 bytes a corner plus 4 a face.

/// Vertex indices per face, in file order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BFaces {
    /// Every face's corners, back to back.
    verts: Vec<u32>,
    /// `starts[f] .. starts[f + 1]` is face `f`; always ends with `verts.len()`.
    starts: Vec<u32>,
}

impl BFaces {
    pub fn len(&self) -> usize {
        self.starts.len().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total corners over all faces.
    pub fn corners(&self) -> usize {
        self.verts.len()
    }

    /// Face `f`'s vertex indices, or `&[]` past the end.
    pub fn get(&self, f: usize) -> &[u32] {
        match (self.starts.get(f), self.starts.get(f + 1)) {
            (Some(&a), Some(&b)) => &self.verts[a as usize..b as usize],
            _ => &[],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &[u32]> + '_ {
        (0..self.len()).map(|f| self.get(f))
    }

    /// Append one face.
    pub fn push(&mut self, face: &[u32]) {
        if self.starts.is_empty() {
            self.starts.push(0);
        }
        self.verts.extend_from_slice(face);
        self.starts.push(self.verts.len() as u32);
    }

    /// Room for `faces` more faces of `corners` more corners in total.
    pub fn reserve(&mut self, faces: usize, corners: usize) {
        self.starts.reserve(faces + 1);
        self.verts.reserve(corners);
    }

    /// Where face `f` starts among the corners (its first loop).
    pub fn start(&self, f: usize) -> usize {
        self.starts.get(f).copied().unwrap_or(0) as usize
    }
}

impl std::ops::Index<usize> for BFaces {
    type Output = [u32];

    fn index(&self, f: usize) -> &[u32] {
        self.get(f)
    }
}

impl FromIterator<Vec<u32>> for BFaces {
    fn from_iter<I: IntoIterator<Item = Vec<u32>>>(iter: I) -> Self {
        let mut out = Self::default();
        iter.into_iter().for_each(|f| out.push(&f));
        out
    }
}

impl<'a> FromIterator<&'a [u32]> for BFaces {
    fn from_iter<I: IntoIterator<Item = &'a [u32]>>(iter: I) -> Self {
        let mut out = Self::default();
        iter.into_iter().for_each(|f| out.push(f));
        out
    }
}

impl From<Vec<Vec<u32>>> for BFaces {
    fn from(faces: Vec<Vec<u32>>) -> Self {
        faces.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_faces_and_costs_one_array() {
        let f: BFaces = vec![vec![0, 1, 2], vec![2, 3, 4, 5]].into();
        assert_eq!((f.len(), f.corners()), (2, 7));
        assert_eq!(f.get(0), [0, 1, 2]);
        assert_eq!(&f[1], [2, 3, 4, 5]);
        assert_eq!(f.start(1), 3, "face 1's first loop");
        assert_eq!(f.get(2), [] as [u32; 0], "past the end");
        assert_eq!(f.iter().map(<[u32]>::len).sum::<usize>(), 7);
        let mut g = BFaces::default();
        g.push(&[0, 1, 2]);
        g.push(&[2, 3, 4, 5]);
        assert_eq!(g, f);
    }
}
