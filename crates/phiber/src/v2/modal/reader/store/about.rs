//! The manual's front page: what Blender is and who uses it —
//! `manual/getting_started/about/index.rst`.

use super::super::set::{BReaderBlock::*, BReaderDoc};

pub static DOC: BReaderDoc = BReaderDoc {
    slug: "manual",
    icon: "manual",
    title: "About Blender",
    source: "Blender Manual — Getting Started ▸ About Blender",
    blocks: &[
        Para("Blender is the free and open-source 3D creation suite. It supports the entirety of the 3D pipeline: modeling, rigging, animation, simulation, rendering, compositing, motion tracking and video editing."),
        Para("Blender provides a consistent experience across Linux, macOS, and Windows operating systems."),
        Heading("Who uses Blender?"),
        Para("Blender has a wide variety of tools making it suitable for almost any sort of media production. Professionals, hobbyists, and studios around the world use it for creating animations, game assets, motion graphics, TV shows, concept art, story-boarding, commercials, and feature films."),
        Heading("Key Features"),
        Item("Blender is a fully integrated 3D content creation suite, offering a broad range of essential tools, including Modeling, Rendering, Animation & Rigging, Video Editing, VFX, Compositing, Texturing, and many types of Simulations."),
        Item("It is cross platform, with a GUI that is uniform on all major platforms (and customizable with Python scripts)."),
        Item("It has a high-quality 3D architecture, enabling fast and efficient creation workflow."),
        Item("It boasts active community support. See the User Communities page for an extensive list of sites."),
        Item("It can be installed into and run from any directory without modifying the system."),
        Heading("Learning Blender"),
        Para("Blender makes it possible to perform a wide range of tasks, and it may seem daunting when first trying to grasp the basics. However, with a bit of motivation and the right learning material, it is possible to familiarize yourself with Blender after a few hours of practice."),
        Para("This manual is a good start, though it serves more as a reference. There are also many online video tutorials from specialized websites."),
        Para("Despite everything Blender can do, it remains a tool. Great artists do not create masterpieces by pressing buttons or manipulating brushes, but by learning and practicing subjects such as human anatomy, composition, lighting, animation principles, etc."),
        Para("3D creation software such as Blender have an added technical complexity and jargon associated with the underlying technologies. Terms like UV maps, materials, shaders, meshes, and \"subdivs\" are the media of the digital artist, and understanding them, even broadly, will help you to use Blender to its best."),
        Para("So keep reading this manual, learn the great tool that Blender is, and keep your mind open to other artistic and technological areas — and you, too, can become a great artist."),
        Heading("Phiber and this manual"),
        Para("Phiber follows Blender's window system, editors and workflows part for part, so Blender's manual is the reference for what you see here: the same workspaces, the same editor types, the same menus. Where Phiber differs — its own brand, its activity bar, its welcome window — this reader says so."),
        Heading("Further Reading"),
        Item("Help System — tooltips, the Help menu and Save System Info."),
        Item("The Blender Community — independent sites, support, development and chat."),
        Item("License — Blender is free software; what you make with it is yours."),
    ],
};
