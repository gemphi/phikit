//! `--help` and `--version`, and the two `help` listings `-E` and `-F`
//! print instead of rendering.
//!
//! The text is built from [`set::ARG`], so a flag added to the table shows
//! up here without a second edit — the "one owner per treatment" rule
//! applied to the command line.

use super::set::{ARG, BArgTake, BRenderQuality, ENGINE, FORMAT};

/// What the binary calls itself in its own usage line.
pub const NAME: &str = "phiber";

/// `--version`.
pub fn version() -> String {
    format!("{NAME} {}", env!("CARGO_PKG_VERSION"))
}

/// `--help`: the usage line, then one row per flag, then what differs from
/// Blender — stated here rather than left for someone to discover.
pub fn help() -> String {
    let mut out = String::new();
    out.push_str(&format!("Usage: {NAME} [args ...] [file] [args ...]\n\n"));
    out.push_str("Render Options:\n");
    for spec in ARG {
        let names = match (spec.short.is_empty(), spec.long.is_empty()) {
            (false, false) => format!("{}, {}", spec.short, spec.long),
            (true, false) => format!("    {}", spec.long),
            _ => spec.short.to_string(),
        };
        let names = match spec.take {
            BArgTake::Word => format!("{names} <value>"),
            BArgTake::Nothing => names,
        };
        out.push_str(&format!("  {names:<34}{}\n", spec.help));
    }
    out.push_str(&format!(
        "\nDefaults that differ from Blender:\n  \
         The engine is {} unless -E names another.\n  \
         The sample count is at least {}; --draft lifts that floor.\n",
        BRenderQuality::ENGINE,
        BRenderQuality::SAMPLE_FLOOR
    ));
    out
}

/// `-E help`.
pub fn engine_list() -> String {
    format!("Blender Engine Listing:\n{}\n", ENGINE.iter().map(|e| format!("\t{e}")).collect::<Vec<_>>().join("\n"))
}

/// `-F help`.
pub fn format_list() -> String {
    format!("Blender File Format Listing:\n{}\n", FORMAT.iter().map(|f| format!("\t{f}")).collect::<Vec<_>>().join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_help_lists_every_flag_and_says_what_differs() {
        let text = help();
        for spec in ARG {
            for name in [spec.short, spec.long].into_iter().filter(|n| !n.is_empty()) {
                assert!(text.contains(name), "{name} is missing from --help");
            }
            assert!(text.contains(spec.help), "{spec:?}'s help line is missing");
        }
        assert!(text.contains("PHIRAY"), "the engine default is stated");
        assert!(text.contains("2048"), "the sample floor is stated");
        assert!(text.starts_with("Usage: phiber"));
    }

    #[test]
    fn a_flag_that_takes_a_word_says_so_in_the_listing() {
        let text = help();
        assert!(text.contains("-f, --render-frame <value>"), "{text}");
        assert!(text.contains("-b, --background  "), "a bare flag takes no <value>");
        assert!(text.contains("    --samples <value>"), "a long-only flag is indented into the column");
    }

    #[test]
    fn the_two_help_listings_name_what_they_accept() {
        assert!(engine_list().contains("PHIRAY") && engine_list().contains("CYCLES"));
        assert!(format_list().contains("PNG") && format_list().contains("OPEN_EXR"));
        // Phiray is listed first, because it is the default.
        let listing = engine_list();
        assert!(listing.find("PHIRAY") < listing.find("CYCLES"));
    }

    #[test]
    fn the_version_names_the_binary_and_its_crate_version() {
        let text = version();
        assert!(text.starts_with("phiber "));
        assert!(text.len() > "phiber ".len(), "a version follows the name");
    }
}
