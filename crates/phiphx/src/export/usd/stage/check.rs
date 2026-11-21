//! A parser-level check that a layer from [`UsdStage`](super::UsdStage) is
//! well formed. It is not a USD parser: it reads the layout this writer
//! produces (one brace per line, one array per line) and answers what a
//! test needs, without `pxr`. When `pxr` is at hand, the integration test
//! `usd_stage_truth` also loads the layer with it.
use std::collections::{BTreeMap, HashSet};

/// What a layer holds, counted while it was checked.
#[derive(Debug, Default, PartialEq)]
pub struct UsdReport {
    /// Prims by type name ("Mesh", "Xform", ...).
    pub prim: BTreeMap<String, usize>,
    /// Sum of every mesh's `faceVertexCounts`.
    pub corner: usize,
    /// Number of faces (entries of `faceVertexCounts`) over all meshes.
    pub face: usize,
    /// Triangles over all meshes (a face of `n` corners is `n - 2`).
    pub triangle: usize,
}

impl UsdReport {
    /// How many prims of `kind`.
    pub fn count(&self, kind: &str) -> usize {
        self.prim.get(kind).copied().unwrap_or(0)
    }
}

/// The checker. A unit type: [`UsdCheck::run`] is the entry.
pub struct UsdCheck;

impl UsdCheck {
    /// Check `text` and count what is in it, or say what is wrong:
    ///
    /// - the first line is `#usda 1.0`;
    /// - `{ [ (` are balanced outside strings and `@asset@` literals;
    /// - every prim name is a valid identifier and unique among siblings;
    /// - per mesh: `sum(faceVertexCounts) == len(faceVertexIndices)`, every
    ///   index is below `len(points)`, and `normals` / `primvars:st` /
    ///   `primvars:displayColor` have the count their interpolation needs;
    /// - every `GeomSubset` index is below the mesh's face count.
    pub fn run(text: &str) -> Result<UsdReport, String> {
        if text.lines().next() != Some("#usda 1.0") {
            return Err("missing `#usda 1.0` header".into());
        }
        Self::balance(text)?;
        let mut report = UsdReport::default();
        let mut sibling: Vec<HashSet<String>> = vec![HashSet::new()];
        let mut pending = None;
        let mut mesh = MeshCount::default();
        let mut in_mesh = 0usize;
        // The layer metadata (up to its closing `)`) holds a dictionary whose braces are not prims.
        let body = text.lines().position(|l| l == ")").map_or(0, |i| i + 1);
        for (n, line) in text
            .lines()
            .enumerate()
            .map(|(i, l)| (i + 1, l.trim()))
            .skip(body)
        {
            let at = |m: String| format!("line {n}: {m}");
            match (Self::prim(line), line) {
                (Some((kind, name)), _) => {
                    if !Self::is_id(name) {
                        return Err(at(format!("`{name}` is not a valid prim name")));
                    }
                    *report.prim.entry(kind.to_string()).or_default() += 1;
                    pending = Some((kind.to_string(), name.to_string()));
                }
                (None, "{") => {
                    let (kind, name) = pending
                        .take()
                        .ok_or_else(|| at("block without a prim".into()))?;
                    if !sibling.last_mut().is_some_and(|s| s.insert(name.clone())) {
                        return Err(at(format!("`{name}` twice among siblings")));
                    }
                    sibling.push(HashSet::new());
                    if kind == "Mesh" {
                        in_mesh = sibling.len();
                        mesh = MeshCount::default();
                    }
                }
                (None, "}") => {
                    if in_mesh == sibling.len() {
                        mesh.verify(&mut report).map_err(at)?;
                        in_mesh = 0;
                    }
                    sibling.pop();
                }
                (None, _) if in_mesh > 0 => mesh.read(line).map_err(at)?,
                _ => {}
            }
        }
        Ok(report)
    }

    /// `def Type "name"` as (Type, name).
    fn prim(line: &str) -> Option<(&str, &str)> {
        let rest = line
            .strip_prefix("def ")
            .or_else(|| line.strip_prefix("over "))
            .or_else(|| line.strip_prefix("class "))?;
        let (kind, name) = rest.split_once(' ')?;
        Some((kind, name.trim().strip_prefix('"')?.strip_suffix('"')?))
    }

    fn is_id(s: &str) -> bool {
        let mut c = s.chars();
        c.next()
            .is_some_and(|f| f.is_ascii_alphabetic() || f == '_')
            && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
    }

    /// Brackets balance, ignoring the inside of `"..."` and `@...@`.
    fn balance(text: &str) -> Result<(), String> {
        let mut stack = Vec::new();
        let (mut quote, mut asset, mut escape) = (false, false, false);
        for (n, line) in text.lines().enumerate() {
            for ch in line.chars() {
                if quote {
                    match (escape, ch) {
                        (true, _) => escape = false,
                        (false, '\\') => escape = true,
                        (false, '"') => quote = false,
                        _ => {}
                    }
                    continue;
                }
                match ch {
                    '"' => quote = true,
                    '@' => asset = !asset,
                    _ if asset => {}
                    '{' | '[' | '(' => stack.push(ch),
                    '}' | ']' | ')' => {
                        let open = stack.pop();
                        if open
                            != Some(match ch {
                                '}' => '{',
                                ']' => '[',
                                _ => '(',
                            })
                        {
                            return Err(format!("line {}: `{ch}` closes {open:?}", n + 1));
                        }
                    }
                    _ => {}
                }
            }
            if quote || asset {
                return Err(format!("line {}: string or asset left open", n + 1));
            }
        }
        if stack.is_empty() {
            Ok(())
        } else {
            Err(format!("{} block(s) left open", stack.len()))
        }
    }
}

/// The counts of the mesh being read.
#[derive(Default)]
struct MeshCount {
    point: usize,
    count: Vec<usize>,
    index: usize,
    max_index: usize,
    /// (attribute, entries, per corner?)
    primvar: Vec<(String, usize, bool)>,
    subset_max: usize,
}

impl MeshCount {
    fn read(&mut self, line: &str) -> Result<(), String> {
        let Some((decl, value)) = line.split_once(" = ") else {
            return Ok(());
        };
        let name = decl.rsplit(' ').next().unwrap_or("");
        let body = value.split(" (interpolation").next().unwrap_or(value);
        match name {
            "points" => self.point = body.matches('(').count(),
            "faceVertexCounts" => self.count = Self::ints(body)?,
            "faceVertexIndices" => {
                let v = Self::ints(body)?;
                self.index = v.len();
                self.max_index = v.iter().copied().max().map_or(0, |m| m + 1);
            }
            "normals" | "primvars:st" | "primvars:displayColor" => {
                let corner = value.contains("faceVarying");
                self.primvar
                    .push((name.to_string(), body.matches('(').count(), corner));
            }
            "indices" => {
                self.subset_max = self
                    .subset_max
                    .max(Self::ints(body)?.iter().copied().max().map_or(0, |m| m + 1))
            }
            _ => {}
        }
        Ok(())
    }

    fn ints(body: &str) -> Result<Vec<usize>, String> {
        let inner = body.trim().trim_start_matches('[').trim_end_matches(']');
        if inner.trim().is_empty() {
            return Ok(Vec::new());
        }
        inner
            .split(',')
            .map(|s| s.trim().parse::<usize>().map_err(|e| format!("`{s}`: {e}")))
            .collect()
    }

    fn verify(&self, report: &mut UsdReport) -> Result<(), String> {
        let corner: usize = self.count.iter().sum();
        if corner != self.index {
            return Err(format!(
                "sum(faceVertexCounts) = {corner} but {} faceVertexIndices",
                self.index
            ));
        }
        if self.max_index > self.point {
            return Err(format!(
                "faceVertexIndices reach {} but there are {} points",
                self.max_index, self.point
            ));
        }
        for (name, n, per_corner) in &self.primvar {
            let want = if *per_corner { corner } else { self.point };
            if *n != want {
                return Err(format!(
                    "{name} has {n} entries, its interpolation needs {want}"
                ));
            }
        }
        if self.subset_max > self.count.len() {
            return Err(format!(
                "a GeomSubset reaches face {} of {}",
                self.subset_max,
                self.count.len()
            ));
        }
        report.corner += corner;
        report.face += self.count.len();
        report.triangle += self
            .count
            .iter()
            .map(|c| c.saturating_sub(2))
            .sum::<usize>();
        Ok(())
    }
}
