//! Every part reaches its view the same way, and the part table knows every
//! part.
//!
//! phiber has no `BPart` trait — `structure.md:96` declares one and no such
//! file was ever written. What it has instead is a convention that does the
//! same job: every part folder's `mod.rs` defines a `B<Name>` with `::new(host)`
//! and `impl BViewItem for B<Name>`, and `BPartItem::body` dispatches to
//! `B<Name>::new(h).build(s)`.
//!
//! A convention with nothing checking it is a convention until someone is in a
//! hurry. Five of the thirty-seven variants — the whole file browser —
//! dispatched to free functions instead, and nothing said so, because the only
//! thing that would have noticed was a person reading the match arms. These
//! tests are that reader.
//!
//! They are deliberately source-reading rather than type-level. A trait bound
//! would be the better guard and wants the trait to exist first; until it does,
//! the dispatcher's own text is the only place the shape is visible.

use std::collections::BTreeSet;

const PART_MOD: &str = include_str!("../src/v2/part/mod.rs");

/// The body of `BPartItem::body`, which is the dispatcher.
fn dispatcher() -> &'static str {
    let from = PART_MOD.find("fn body(").expect("the part dispatcher");
    let rest = &PART_MOD[from..];
    let end = rest.find("\n    }\n").map_or(rest.len(), |i| i + 6);
    &rest[..end]
}

/// Every `BPartType::<Variant> =>` arm, with what it dispatches to.
///
/// Or-patterns share one arm — `ToolProps | BPartType::AdjustLastOp => …` —
/// and every name in one is dispatched, so each is recorded against the arm
/// they share. A parser that missed that reported `ToolProps` as unreachable
/// when it is reached; the first version of this did exactly that.
fn arms() -> Vec<(String, String)> {
    let body = dispatcher();
    let mut pending: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for piece in body.split("BPartType::").skip(1) {
        match piece.split_once(" => ") {
            // A name with no arm of its own is the left side of an or-pattern.
            None => {
                let name = piece.split('|').next().unwrap_or("").trim();
                if !name.is_empty() && !name.contains('\n') {
                    pending.push(name.to_string());
                }
            }
            Some((variant, rest)) => {
                let variant = variant.trim();
                if variant.contains('\n') || variant.contains('(') {
                    pending.clear();
                    continue;
                }
                let to: String = rest.lines().next().unwrap_or("").trim().to_string();
                for shared in pending.drain(..) {
                    out.push((shared, to.clone()));
                }
                out.push((variant.to_string(), to));
            }
        }
    }
    out
}

#[test]
fn every_part_reaches_its_view_through_the_one_interface() {
    // `B<Name>::new(h).build(s)` — or one of the two deliberate exceptions.
    //
    // `Scrubbing` is a no-op: the scrub strip is painted by the time canvas,
    // not built as a view, so the part exists to hold the slot. Anything else
    // that does not call `.build(` is a part that quietly stopped being one.
    let exceptions: BTreeSet<&str> = ["Scrubbing"].into_iter().collect();
    let mut stragglers = Vec::new();
    for (variant, to) in arms() {
        if exceptions.contains(variant.as_str()) {
            continue;
        }
        if !to.contains(".build(") && !to.contains("{") {
            stragglers.push(format!("{variant} => {to}"));
        }
    }
    assert!(
        stragglers.is_empty(),
        "these parts do not go through `BViewItem`:\n  {}\n\
         Give each a `B<Name>` with `::new(host)` and an `impl BViewItem`, as every other part has.",
        stragglers.join("\n  "),
    );
}

#[test]
fn the_file_browsers_parts_are_parts_like_the_rest() {
    // Named, because this is where the interface actually broke: five arms in
    // one corner, all reaching `file::<module>::<free fn>(s)`.
    let body = dispatcher();
    for part in ["FileNav", "FileRefresh", "PathBar", "SourceList", "TypeFilter"] {
        let arm = body
            .split(&format!("BPartType::{part} => "))
            .nth(1)
            .unwrap_or_else(|| panic!("{part} has no arm"))
            .lines()
            .next()
            .unwrap_or("");
        assert!(
            arm.contains(".build("),
            "{part} dispatches to `{}`, not through `BViewItem`",
            arm.trim(),
        );
    }
}

#[test]
fn each_of_those_five_has_a_type_of_its_own() {
    // The impls themselves, so the guard fails on the file that lost one
    // rather than only on the dispatcher.
    for (source, wanted) in [
        (include_str!("../src/v2/part/file/nav.rs"), ["BFileNav", "BFileRefresh"].as_slice()),
        (include_str!("../src/v2/part/file/path_bar.rs"), ["BPathBar", "BTypeFilter"].as_slice()),
        (include_str!("../src/v2/part/file/sidebar.rs"), ["BSourceList"].as_slice()),
    ] {
        for name in wanted {
            assert!(
                source.contains(&format!("impl BViewItem for {name}"))
                    || source.contains(&format!("impl crate::v2::view::BViewItem for {name}")),
                "{name} has no `BViewItem` impl",
            );
            assert!(
                source.contains(&format!("impl {name} {{")),
                "{name} has no `::new(host)` of its own",
            );
        }
    }
}

#[test]
fn the_dispatcher_covers_every_variant_the_enum_declares() {
    // The enum and the dispatcher are two hand-kept lists, and the compiler
    // only enforces one direction (a missing arm is an error). This is the
    // other direction stated out loud: every variant is reachable.
    let from = PART_MOD.find("pub enum BPartType {").expect("the part enum");
    let rest = &PART_MOD[from..];
    let end = rest.find("\n}\n").expect("the enum's end");
    let variants: BTreeSet<String> = rest[..end]
        .lines()
        .filter_map(|line| {
            let line = line.split("//").next()?.trim();
            let name = line.strip_suffix(',')?;
            (!name.is_empty() && name.chars().next()?.is_ascii_uppercase() && !name.contains(' '))
                .then(|| name.to_string())
        })
        .collect();
    assert!(variants.len() > 30, "only found {} variants — the parse is wrong", variants.len());

    let dispatched: BTreeSet<String> = arms().into_iter().map(|(v, _)| v).collect();
    let missing: Vec<&String> = variants.difference(&dispatched).collect();
    assert!(missing.is_empty(), "variants with no dispatcher arm: {missing:?}");
}

#[test]
fn the_part_table_covers_every_variant() {
    // `part/set.rs` held 28 entries for 37 variants and nothing noticed,
    // because nothing reads it and nothing compared it to the enum. Both are
    // still true of the reading; this is the comparing.
    use phiber::v2::part::{BPartType, SET};
    // `BPartType` is not `Ord`, so compare by name rather than adding a
    // derive to the enum for a test's convenience.
    let listed: BTreeSet<String> = SET.iter().map(|spec| format!("{:?}", spec.kind)).collect();
    let all: BTreeSet<String> = BPartType::ALL.iter().map(|k| format!("{k:?}")).collect();
    let missing: Vec<&String> = all.difference(&listed).collect();
    assert!(missing.is_empty(), "parts with no entry in `part/set.rs`: {missing:?}");
    assert_eq!(SET.len(), BPartType::ALL.len(), "and no entry twice");
}

#[test]
fn a_parts_label_exists_once() {
    // It used to exist twice — the enum's match and a string per `set.rs`
    // entry — with nothing comparing them. A spec asks the enum now.
    use phiber::v2::part::{BPartType, SET};
    for spec in SET {
        assert_eq!(spec.label(), spec.kind.label(), "{:?}", spec.kind);
    }
    assert!(!BPartType::ALL.is_empty());
}

#[test]
fn the_enum_and_its_all_slice_agree() {
    // `ALL` is hand-kept beside the enum because Rust cannot walk variants.
    // This is the thing that makes forgetting a line fail.
    use phiber::v2::part::BPartType;
    let from_source: BTreeSet<String> = {
        let from = PART_MOD.find("pub enum BPartType {").expect("the enum");
        let rest = &PART_MOD[from..];
        let end = rest.find("\n}\n").expect("its end");
        rest[..end]
            .lines()
            .filter_map(|line| {
                let name = line.split("//").next()?.trim().strip_suffix(',')?;
                (!name.is_empty()
                    && name.chars().next()?.is_ascii_uppercase()
                    && !name.contains(' '))
                .then(|| name.to_string())
            })
            .collect()
    };
    assert_eq!(
        from_source.len(),
        BPartType::ALL.len(),
        "the enum declares {} variants and `ALL` lists {}",
        from_source.len(),
        BPartType::ALL.len(),
    );
}

#[test]
fn every_host_is_a_host() {
    // The host half. A host is a module, so each has a zero-sized type in
    // `host/kind/` implementing `BHost` — which is what makes the folder's
    // `SLUG`, `SPEC` and `REGIONS` a compiler-checked shape rather than a
    // convention held up by an exhaustive match on two of the three.
    use phiber::v2::host::BHostType;
    let kind = include_str!("../src/v2/host/kind/mod.rs");
    let listed: BTreeSet<String> = kind
        .split("hosts! {")
        .nth(1)
        .expect("the host list")
        .lines()
        .filter_map(|l| l.split("=>").next()?.trim().strip_suffix("").map(str::to_string))
        .filter(|n| !n.is_empty() && n.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
        .collect();
    // Every variant but `Empty`, which is a placeholder with no folder.
    let expected: BTreeSet<String> = BHostType::ALL
        .iter()
        .map(|h| format!("{h:?}"))
        .filter(|n| n != "Empty")
        .collect();
    let missing: Vec<&String> = expected.difference(&listed).collect();
    assert!(missing.is_empty(), "hosts with no `BHost` implementation: {missing:?}");
}

