//! An asset: a file an app ships with, such as an icon, a font or an
//! image, found by its path. gpui's `AssetSource` is the model: the app
//! hands over one source, and whatever draws asks it by path.

/// Where an app's files come from: a folder on disk, the files built into
/// the program, or the web.
pub trait PAsset {
    /// The bytes of the file at `path`, or `None` when there is none.
    fn load(&self, path: &str) -> Option<Vec<u8>>;

    /// The paths of the files in `folder`. A source that cannot list
    /// answers with none.
    fn list(&self, _folder: &str) -> Vec<String> {
        Vec::new()
    }

    /// The pictures of the app that move, where it has any: a name the
    /// film holds is drawn from it, and from no file.
    fn film(&self) -> Option<super::PFilm> {
        None
    }
}
