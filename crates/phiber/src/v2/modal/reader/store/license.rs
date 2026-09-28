//! blender.org's License page — `blender.org/about/license`.

use super::super::set::{BReaderBlock::*, BReaderDoc};

pub static DOC: BReaderDoc = BReaderDoc {
    slug: "license",
    icon: "url",
    title: "License",
    source: "blender.org — About ▸ License",
    blocks: &[
        Para("Blender is Free Software. Free to Use. Free to Change. Free to Share. Free to Sell Your Work."),
        Heading("The Software"),
        Para("Blender is released under the GNU General Public License (GPL, or \"free software\"). This license grants people a number of freedoms: you are free to use Blender, for any purpose; you are free to distribute Blender; you can study how Blender works and change it; you can distribute changed versions of Blender. The GPL strictly aims at protecting these freedoms, requiring everyone to share their modifications when they also share the software in public. That aspect is commonly referred to as Copyleft."),
        Para("The Blender Foundation and its projects are dedicated to keeping Blender free software."),
        Heading("License Details"),
        Para("The source code developed at blender.org typically carries \"GNU GPL Version 2 or later\" licensing. However, certain modules — such as the Cycles rendering engine — use more permissive licenses like Apache 2.0."),
        Para("Blender incorporates numerous external libraries with varying licenses: Python (Python License), Bullet (Zlib License), Libmv (MIT License), and OSL (BSD License). All these components remain compatible under \"GNU GPL Version 3 or later,\" which applies to all Blender binary distributions."),
        Heading("Add-ons (Python scripts)"),
        Para("Blender's Python API forms an integral component. The GPL accordingly requires published scripts to use GPL-compliant licensing. Scripts may be sold, though only the distribution service itself generates revenue — customers receive identical GPL terms as Blender itself."),
        Heading("License Compliance"),
        Para("The Foundation prioritizes dialogue when GPL violations arise, seeking mutually acceptable resolutions before considering legal action. Under specific circumstances, Blender functions alongside proprietary software. The Foundation offers guidance on compliance and best practices."),
        Heading("Your Artwork"),
        Para("What you create with Blender is your sole property. All artwork — images, films, .blend files — remains entirely yours. Commercial use by studios, game artists, scientists, and students is unrestricted and protected by Blender's GPL license."),
        Heading("Privacy and Internet access"),
        Para("Blender requires no registration and no connection to the internet for basic operation. Optional add-ons may access the internet; these remain disabled by default and require explicit user permission."),
        Heading("Phiber"),
        Para("Phiber is built on Blender's design and file format, and on the GPUI framework from the Zed editor. Its own source is licensed under the terms shown on the Phiber website; what you make with it is yours, as it is with Blender."),
    ],
};
