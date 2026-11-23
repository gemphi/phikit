use crate::button::{ButtonState, NSButton};
use crate::NSRect;
use std::sync::Arc;

#[test]
fn button_state_default_is_idle() {
    assert_eq!(ButtonState::default(), ButtonState::Idle);
}

#[test]
fn button_starts_in_idle_state() {
    let btn = NSButton::new(NSRect::sized(100.0, 30.0));
    assert_eq!(btn.interaction_state(), ButtonState::Idle);
}

#[test]
fn handle_mouse_entered_transitions_to_hover() {
    let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));
    btn.handle_mouse_entered();
    assert_eq!(btn.interaction_state(), ButtonState::Hover);
}

#[test]
fn handle_mouse_exited_transitions_to_idle() {
    let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));
    btn.handle_mouse_entered();
    btn.handle_mouse_exited();
    assert_eq!(btn.interaction_state(), ButtonState::Idle);
}

#[test]
fn handle_mouse_down_transitions_to_pressed() {
    let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));
    btn.handle_mouse_entered();
    btn.handle_mouse_down();
    assert_eq!(btn.interaction_state(), ButtonState::Pressed);
}

#[test]
fn handle_mouse_up_inside_transitions_to_hover() {
    let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));
    btn.handle_mouse_entered();
    btn.handle_mouse_down();
    btn.handle_mouse_up(true);
    assert_eq!(btn.interaction_state(), ButtonState::Hover);
}

#[test]
fn handle_mouse_up_outside_transitions_to_idle() {
    let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));
    btn.handle_mouse_entered();
    btn.handle_mouse_down();
    btn.handle_mouse_up(false);
    assert_eq!(btn.interaction_state(), ButtonState::Idle);
}

#[test]
fn handle_mouse_up_inside_fires_action() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));
    btn.set_action(move || {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    btn.handle_mouse_entered();
    btn.handle_mouse_down();
    btn.handle_mouse_up(true);

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn handle_mouse_up_outside_does_not_fire_action() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));
    btn.set_action(move || {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    btn.handle_mouse_entered();
    btn.handle_mouse_down();
    btn.handle_mouse_up(false);

    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[test]
fn button_state_background_color_resolves_from_theme() {
    // Each state should resolve to a color from the theme (not transparent/zero)
    let idle_color = ButtonState::Idle.background_color();
    let hover_color = ButtonState::Hover.background_color();
    let pressed_color = ButtonState::Pressed.background_color();

    // All colors should have non-zero alpha (they are real theme colors, not transparent)
    assert!(idle_color.a > 0.0, "Idle color should have non-zero alpha");
    assert!(hover_color.a > 0.0, "Hover color should have non-zero alpha");
    assert!(pressed_color.a > 0.0, "Pressed color should have non-zero alpha");
}

#[test]
fn current_background_color_matches_state() {
    let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));

    // Idle state
    let idle_color = btn.current_background_color();
    assert_eq!(idle_color, ButtonState::Idle.background_color());

    // Hover state
    btn.handle_mouse_entered();
    let hover_color = btn.current_background_color();
    assert_eq!(hover_color, ButtonState::Hover.background_color());

    // Pressed state
    btn.handle_mouse_down();
    let pressed_color = btn.current_background_color();
    assert_eq!(pressed_color, ButtonState::Pressed.background_color());
}
