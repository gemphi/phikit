//! A digest of every evaluated object's geometry, so two reads can be proved
//! to agree without comparing a quarter-million vertices by eye.
//!
//! Written for `modifier/prefix.rs`, where the mesh a stack reaches before its
//! Armature is applied by one consumer and handed to the other: the saving is
//! only real if the evaluated scene is unchanged, and "the vertex counts
//! match" is not that proof — the same count with the prefix applied twice in
//! different orders would pass it. So this hashes the positions themselves:
//!
//! ```text
//! cargo run --release -p phiphx --example dump_eval -- f.blend > on.txt
//! PHIPHX_NOSHARE=1 cargo run --release -p phiphx --example dump_eval -- f.blend > off.txt
//! diff on.txt off.txt     # must be empty
//! ```
//!
//! Measured identical over every fixture with geometry in it —
//! FLORIS_BOLDRIOD_RIG, THYMUS-BOLDROID-WAVE-003, MATERIALS-BSDF,
//! PROCEDURAL, nla_cube, constraints and LIGHTS: 310,559 vertices over 613
//! objects. It is a plain FNV-1a over the little-endian bytes of each
//! position, in object-name order, so a run is comparable with a run on
//! another machine as long as both were built the same way.
//!
//! Positions only: this exists to catch a *geometry* difference between two
//! evaluation paths. The UVs, materials and smooth flags ride the same
//! stencils and the truth tests (`tests/blender_truth.rs`, `tests/skinner.rs`)
//! compare them against Blender's own output.

fn digest(mesh: &[[f32; 3]]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for position in mesh {
        for channel in position {
            for byte in channel.to_le_bytes() {
                h ^= byte as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
        }
    }
    h
}

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_none() {
        eprintln!("usage: dump_eval <file.blend> [more.blend ...]");
        std::process::exit(2);
    }
    for path in args {
        let model = match phiphx::Phx::open(std::path::Path::new(&path)) {
            Ok(model) => model,
            Err(e) => {
                eprintln!("{path}: {e:?}");
                std::process::exit(1);
            }
        };
        // Sorted by name, not by index: an object's index is an artefact of
        // the read order, and a reordering that kept every mesh intact is not
        // the difference this is looking for.
        let mut rows: Vec<String> = model
            .objects
            .iter()
            .map(|o| format!("{}\t{}\t{:016x}", o.name, o.mesh.vertices.len(), digest(&o.mesh.vertices)))
            .collect();
        rows.sort();
        println!("# {path}\t{} objects", rows.len());
        for row in rows {
            println!("{row}");
        }
    }
}
