//! The activity bar's secondary strip is the card shade, flat.
//!
//! It is a `BCard::panel` filling `pane-Center-Activity` edge to edge, and it
//! was painted with the theme's sweep like any card laid on a surface. But a
//! sweep reads as a raised card BECAUSE there is a surface under it, and here
//! there is none to see: the panel is the whole region. So the sweep's top stop
//! was not a highlight, it was a second shade running down the strip's own
//! edge — and measured, that stop was the exact shade the rail's entry tabs
//! beside it are painted in. The strip read as more rail.
//!
//! Flat, it is the card shade from top to bottom: the same shade as the content
//! it carries, which is the whole of what it is for.
//!
//! Measured, not asserted from the tokens: the tokens were already right —
//! `panel_tone` has said `BSurfaceTone::Card` all along — and what went wrong
//! was how that tone got painted. Only the frame can show that.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{ActivityKey, BState, CtxKey};
use phiber::v2::theme::{BPaletteName, BSurfaceTone, BThemeMode, resolve_ui_style, ui_style_defaults_in};

/// The fills painted at `bounds`, opaque ones only, as `(is_gradient, first
/// lightness, last lightness)`.
fn fills_at(lines: &[String], scale: f32, want: (f32, f32, f32, f32)) -> Vec<(bool, f32, f32)> {
    let mut out = Vec::new();
    for line in lines {
        if !line.starts_with("quad ") {
            continue;
        }
        let mut f = line.split(' ');
        f.next();
        let (Some(origin), Some(sizes)) = (f.next(), f.next()) else { continue };
        let (Some((x, y)), Some((w, h))) = (origin.split_once(','), sizes.split_once('x')) else { continue };
        let num = |v: &str| v.parse::<f32>().unwrap_or(0.0) / scale;
        let (x, y, w, h) = (num(x), num(y), num(w), num(h));
        if (x - want.0).abs() > 2.0 || (y - want.1).abs() > 2.0 || (w - want.2).abs() > 3.0 || (h - want.3).abs() > 3.0 {
            continue;
        }
        let bg = line.split(" bg ").nth(1).unwrap_or("").split(" border ").next().unwrap_or("");
        // Transparent quads are borders, not fills.
        let alphas: Vec<f32> = bg.split("a: ").skip(1).filter_map(|r| r.split(&[',', ' ', '}'][..]).next()?.parse().ok()).collect();
        if alphas.iter().all(|a| *a == 0.0) {
            continue;
        }
        let ls: Vec<f32> = bg
            .split("l: ")
            .skip(1)
            .filter_map(|r| r.split(&[',', ' '][..]).next()?.parse::<f32>().ok())
            .map(|v| v * 100.0)
            .collect();
        if ls.is_empty() {
            continue;
        }
        out.push((bg.starts_with("LinearGradient"), ls[0], *ls.last().unwrap()));
    }
    out
}

#[gpui::test]
async fn the_secondary_strip_is_the_card_shade_flat(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        // A grouping open, so the strip is on screen at all.
        app.state.set(ActivityKey::Group, "Spaces");
        cx.notify();
    });
    cx.run_until_parked();

    let scale = cx.update(|window, _| window.scale_factor());
    let painted = cx.scene_fingerprint();
    let bounds = cx.all_debug_bounds();
    let at = |sel: &str| {
        bounds
            .iter()
            .find(|(s, _)| s == sel)
            .map(|(_, b)| {
                (f32::from(b.origin.x), f32::from(b.origin.y), f32::from(b.size.width), f32::from(b.size.height))
            })
            .unwrap_or_else(|| panic!("{sel} is painted"))
    };
    let strip = at("pane-Center-Activity");
    let rail_entry = at("tab-home-home");
    let region = at("region-Center-Activity");

    let style = resolve_ui_style(ui_style_defaults_in(BPaletteName::DEFAULT, BThemeMode::DEFAULT), &BState::new());
    let card = style.surface(BSurfaceTone::Card).bottom.2;

    let strip_fills = fills_at(&painted, scale, strip);
    assert!(!strip_fills.is_empty(), "the strip paints a fill");
    for (gradient, first, last) in &strip_fills {
        assert!(!gradient, "the strip is flat, not swept: it IS its region, so there is nothing under it to be raised from");
        assert!(
            (first - card).abs() < 0.2 && (last - card).abs() < 0.2,
            "the strip is the card shade ({card:.2}), got {first:.2}..{last:.2}"
        );
    }

    // And it no longer shares a shade with the rail beside it, at any edge.
    // This is the thing that made it read as rail: the sweep's top stop WAS
    // the entry-tab shade, exactly.
    let entry_shades: Vec<f32> = fills_at(&painted, scale, rail_entry).iter().map(|(_, f, _)| *f).collect();
    assert!(!entry_shades.is_empty(), "a rail entry paints a fill");
    for entry in &entry_shades {
        for (_, first, last) in &strip_fills {
            assert!(
                (first - entry).abs() > 0.5 && (last - entry).abs() > 0.5,
                "the strip at {first:.2}..{last:.2} must not meet the rail's entries at {entry:.2}"
            );
        }
    }
    // Nor with the region the two of them stand in.
    let region_shades: Vec<f32> = fills_at(&painted, scale, region).iter().map(|(_, f, _)| *f).collect();
    for ground in &region_shades {
        for (_, first, _) in &strip_fills {
            assert!(
                (first - ground).abs() > 0.5,
                "the strip at {first:.2} must stand clear of its region at {ground:.2}"
            );
        }
    }
}

/// A card that really is laid on a surface keeps its sweep: this is a rule
/// about one place, not a retreat from gradients.
#[gpui::test]
async fn a_card_on_a_surface_keeps_its_sweep(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        // Properties, whose panels are cards on a region.
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    let scale = cx.update(|window, _| window.scale_factor());
    let painted = cx.scene_fingerprint();
    let cards: Vec<(String, (f32, f32, f32, f32))> = cx
        .all_debug_bounds()
        .into_iter()
        .filter(|(s, _)| s.starts_with("card-") && !s.contains("-head") && !s.contains("-content") && !s.contains("-row"))
        .map(|(s, b)| {
            (s, (f32::from(b.origin.x), f32::from(b.origin.y), f32::from(b.size.width), f32::from(b.size.height)))
        })
        .collect();
    assert!(!cards.is_empty(), "Properties draws cards");
    let swept = cards
        .iter()
        .filter(|(_, bounds)| fills_at(&painted, scale, *bounds).iter().any(|(gradient, ..)| *gradient))
        .count();
    assert!(swept > 0, "a card on a region is still swept: {} cards, none swept", cards.len());
}

/// And the rule says which place, in one named question rather than inline.
#[test]
fn the_rule_names_the_one_place_it_applies_to() {
    let src = include_str!("../src/v2/view/draw/card.rs");
    assert!(src.contains("fn panel_fill("), "one function answers it");
    assert!(src.contains("BFill::Floating"), "the floating bar's material — the sidebar's shade");
    assert!(src.contains("BFill::Tone(BSurfaceTone::Card)"), "and the swept card for everywhere else");
    assert!(src.contains("BHostType::Activity"), "scoped to the activity bar");
    assert!(!src.contains("fn panel_tone("), "one function, not a tone and a fill: card.rs is on its line cap");
}
