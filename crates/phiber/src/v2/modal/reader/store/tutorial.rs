//! blender.org's Tutorials page — `blender.org/support/tutorials`.

use super::super::set::{BReaderBlock::*, BReaderDoc};

pub static DOC: BReaderDoc = BReaderDoc {
    slug: "tutorials",
    icon: "tutorials",
    title: "Tutorials",
    source: "blender.org — Support ▸ Tutorials",
    blocks: &[
        Para("Blender is a real community effort, and tutorials are no exception. The most up-to-date tutorials can be found on social media. Look out for the hashtag #b3d on your favorite social platform, such as Mastodon, YouTube, or X."),
        Para("For professional training, real production knowledge and assets, check out Blender Studio. By subscribing to Blender Studio you are also contributing to Blender's development."),
        Heading("Where to start"),
        Item("Blender Fundamentals — the official series that walks through the interface, navigation, selection, modeling, materials and rendering, one short video each."),
        Item("Blender Studio — production training and the source files of the open movies, from character rigging to lighting and compositing."),
        Item("Blender Stack Exchange — a question-and-answer site where a specific problem usually has a specific answer already."),
        Heading("Reading the manual alongside"),
        Para("This manual is a reference rather than a course: when a tutorial names a tool or an editor, the matching manual page explains every option it has. Phiber's Help ▸ Manual opens it here."),
    ],
};
