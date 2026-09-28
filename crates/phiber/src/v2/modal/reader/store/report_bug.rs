//! Reporting a bug — the manual's Help System page and blender.org's Support
//! page, with what a useful report holds.

use super::super::set::{BReaderBlock::*, BReaderDoc};

pub static DOC: BReaderDoc = BReaderDoc {
    slug: "report_bug",
    icon: "bug",
    title: "Report a Bug",
    source: "blender.org — Support ▸ Report a Bug; Blender Manual — Help System",
    blocks: &[
        Para("Sometimes bugs happen, but don't worry! Developers got your back. Please read the best practices guidance before reporting."),
        Para("Blender's reports go to the Blender Bug Tracker at projects.blender.org (registration needed). Phiber's own issues are tracked on the Phiber website — a fault in this window is Phiber's, not Blender's."),
        Heading("Before reporting"),
        Item("Make sure the problem still happens in the latest version."),
        Item("Search the tracker: someone may have reported it already, and their report may hold a workaround."),
        Item("Reduce it: the smallest file and the fewest steps that still show the problem."),
        Heading("What a useful report holds"),
        Term("Title", "One line that says what goes wrong, not \"it crashes\"."),
        Term("Steps", "Numbered, from a fresh start, so anyone can repeat them."),
        Term("Expected and actual", "What should have happened, and what did."),
        Term("System information", "Help ▸ Save System Info writes system-info.txt: the version, build configuration, Python, directories, GPU and driver, and enabled add-ons. Attach it."),
        Term("A file", "The smallest .blend that shows the problem, and a screenshot or short recording when the problem is visual."),
        Heading("Save System Info"),
        Para("This extracts system information which can be useful for including in bug reports, inspecting the configuration, or diagnosing problems. You will be prompted to save a text file called system-info.txt."),
        Para("If you'd like to help fixing bugs, there are plenty of ways to get involved in making Blender better."),
    ],
};
