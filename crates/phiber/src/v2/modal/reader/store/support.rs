//! blender.org's Support page — `blender.org/support`.

use super::super::set::{BReaderBlock::*, BReaderDoc};

pub static DOC: BReaderDoc = BReaderDoc {
    slug: "support",
    icon: "support",
    title: "Support",
    source: "blender.org — Support",
    blocks: &[
        Heading("Documentation"),
        Para("Blender's user manual is available online in several languages and is constantly updated by a worldwide collaboration of volunteers."),
        Item("User Manual"),
        Item("Developer Documentation"),
        Item("Python API Reference"),
        Heading("Tutorials"),
        Para("The most up-to-date tutorials can be found on social media. Look out for the hashtag #b3d on Mastodon, YouTube, Bluesky, or your favorite social network."),
        Para("For professional training, real production knowledge and assets, check out Blender Studio."),
        Heading("FAQ"),
        Para("A list of Frequently Asked Questions covering topics such as using Blender commercially, the GNU GPL license, add-ons usage and more."),
        Heading("Enterprise Support"),
        Para("Canonical offers professional, enterprise-grade support services for the Blender LTS releases. Learn more about this product at ubuntu.com and read the announcement on blender.org."),
        Heading("Community Support"),
        Para("The Blender community offers a wealth of excellent websites with support forums."),
        Item("Blender Stack Exchange"),
        Item("Blender Artists Forum"),
        Item("User Communities"),
        Heading("Report a Bug"),
        Para("Sometimes bugs happen, but don't worry! Developers got your back. Please read the best practices guidance before reporting."),
        Para("If you'd like to help fixing bugs, there are plenty of ways to get involved in making Blender better."),
    ],
};
