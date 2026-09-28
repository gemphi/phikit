//! blender.org's Get Involved page — `blender.org/get-involved`.

use super::super::set::{BReaderBlock::*, BReaderDoc};

pub static DOC: BReaderDoc = BReaderDoc {
    slug: "get_involved",
    icon: "involve",
    title: "Get Involved",
    source: "blender.org — Get Involved",
    blocks: &[
        Para("Join hundreds of contributors worldwide working together towards the freedom to create."),
        Para("Blender is being developed by hundreds of contributors from around the world, including studios and individual artists, professionals and hobbyists, scientists and students, VFX experts and animators, and many more. All united by the desire to have access to a fully free and open-source 3D creation pipeline."),
        Para("The Blender Foundation supports and facilitates these goals. Thanks to donations, the Blender Institute and Blender Studio projects enable a small group of people to work full-time on Blender."),
        Para("For everything else, we depend on volunteers from the online community to achieve this goal. More help is always welcome!"),
        Heading("Development"),
        Para("Volunteers and professionals alike contribute to Blender. This includes developers, technical artists, documenters, translators, designers, and users who test the software and provide feedback."),
        Para("Visit the developer portal to get involved, report bugs, stay up to date, and find resources."),
        Heading("Documentation"),
        Para("Maintaining the Blender Manual up-to-date is a community effort that is key to Blender's adoption."),
        Para("Join the team of volunteers helping to organize and write our complete and freely available end-user documentation. Reach out via the #docs chat channel."),
        Heading("Education"),
        Para("The Blender Education Community shares resources, best practices, and teaching methods, fostering mutual support among educators."),
        Para("Reach out via the #blender-education chat channel to get involved and stay in touch with other Blender trainers."),
        Heading("Infrastructure"),
        Para("The Blender project is supported by an ecosystem of websites. Volunteers who would like to contribute are invited to check out projects.blender.org/infrastructure."),
        Heading("Translations"),
        Para("Blender's user interface and user manual are translated by the community into several languages. If you are interested in getting involved please reach out via the #translations chat channel."),
        Heading("Donations and Sponsors"),
        Para("The Blender Foundation is an independent public-benefit corporation, to facilitate the open source Blender projects. Individuals or companies interested in supporting this financially can contact the foundation or visit the donations page."),
        Para("We also run the Blender Store where all kinds of goodies are sold. The profits are used to cover Blender Foundation expenses and support Blender Institute projects."),
    ],
};
