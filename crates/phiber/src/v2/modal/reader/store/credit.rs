//! Credits — who makes Blender, and what Phiber is built on.

use super::super::set::{BReaderBlock::*, BReaderDoc};

pub static DOC: BReaderDoc = BReaderDoc {
    slug: "credits",
    icon: "heart",
    title: "Credits",
    source: "blender.org — About ▸ Credits; Get Involved",
    blocks: &[
        Para("Blender is being developed by hundreds of contributors from around the world, including studios and individual artists, professionals and hobbyists, scientists and students, VFX experts and animators, and many more. All united by the desire to have access to a fully free and open-source 3D creation pipeline."),
        Para("The Blender Foundation supports and facilitates these goals. Thanks to donations, the Blender Institute and Blender Studio projects enable a small group of people to work full-time on Blender. For everything else, the project depends on volunteers from the online community."),
        Heading("Blender"),
        Term("Blender Foundation", "The independent public-benefit corporation that facilitates the open source Blender projects."),
        Term("Blender Studio", "The production studio whose open movies drive Blender's development, and whose training and assets fund it."),
        Term("Contributors", "Developers, technical artists, documenters, translators, designers, and the users who test the software and provide feedback. The full list of every release's contributors is published with its release notes."),
        Term("Development Fund", "The individuals and companies whose donations pay for full-time development."),
        Heading("Phiber"),
        Term("Design", "Blender's window system, editors, menus and workflows, followed part for part. The manual you are reading is Blender's."),
        Term("Framework", "GPUI, the GPU-accelerated UI framework from the Zed editor, in Phiber's own fork."),
        Term("Renderer", "phiray, Phiber's path tracer, and phiphx, its reader for .blend files."),
        Term("Icons", "Blender's icon set, redrawn as single-colour outlines for Phiber's rails."),
    ],
};
