//! An entry of a level and the set an app supplies: its spaces, its
//! hosts, its parts, its modals. Phiber keeps each of these as an enum
//! (`BSpaceType`, `BHostType`) with a static spec an entry; here an entry
//! is data, so an app adds one without `pui` changing.

use super::PLevel;

/// What every entry of every level has: a name that does not change, which
/// is what a saved layout keeps, and what a person sees for it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PEntry {
    /// The stable name: `view3d`, `modeling`.
    pub slug: String,
    pub label: String,
    /// The name of its icon, or nothing.
    pub icon: String,
}

impl PEntry {
    pub fn new(slug: &str, label: &str, icon: &str) -> PEntry {
        PEntry { slug: slug.to_string(), label: label.to_string(), icon: icon.to_string() }
    }
}

/// The entries of one level an app has, in the order it gave them.
#[derive(Debug, Clone, PartialEq)]
pub struct PSet<T: PLevel> {
    entry: Vec<T>,
}

impl<T: PLevel> Default for PSet<T> {
    fn default() -> Self {
        PSet { entry: Vec::new() }
    }
}

impl<T: PLevel> PSet<T> {
    pub fn new() -> PSet<T> {
        PSet::default()
    }

    /// A set from a table: one entry a line, as `slug`, `label` and
    /// `icon` between tabs, and after them whatever more its level reads
    /// (`PLevel::more`). A line that is empty or starts with `#` is
    /// skipped; the icon may be left out. A name given twice keeps its
    /// first entry.
    pub fn read(text: &str) -> PSet<T> {
        let mut set = PSet::new();
        for line in text.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')) {
            let mut cell = line.split('\t').map(str::trim);
            let slug = cell.next().unwrap_or_default();
            let label = cell.next().unwrap_or_default();
            let mut item = T::of(PEntry::new(slug, label, cell.next().unwrap_or_default()));
            item.more(&cell.collect::<Vec<_>>());
            set.add(item);
        }
        set
    }

    /// Add an entry. `false`, and nothing is added, when the set has
    /// one of that name already or the name is empty.
    pub fn add(&mut self, item: T) -> bool {
        let slug = &item.entry().slug;
        if slug.is_empty() || self.find(slug).is_some() {
            return false;
        }
        self.entry.push(item);
        true
    }

    /// The entry of that name.
    pub fn find(&self, slug: &str) -> Option<&T> {
        self.entry.iter().find(|item| item.entry().slug == slug)
    }

    pub fn find_mut(&mut self, slug: &str) -> Option<&mut T> {
        self.entry.iter_mut().find(|item| item.entry().slug == slug)
    }

    /// Every entry, in order.
    pub fn list(&self) -> &[T] {
        &self.entry
    }
}
