use crate::button::{ButtonState, NSButton};
use crate::view::NSThemeable;
use crate::NSRect;
use proptest::prelude::*;

/// Mouse event variants for property-based testing of the button state machine.
#[derive(Debug, Clone, Copy)]
enum MouseEvent {
    Enter,
    Exit,
    Down,
    Up { cursor_inside: bool },
}

/// Strategy to generate a random mouse event.
fn arb_mouse_event() -> impl Strategy<Value = MouseEvent> {
    prop_oneof![
        Just(MouseEvent::Enter),
        Just(MouseEvent::Exit),
        Just(MouseEvent::Down),
        any::<bool>().prop_map(|cursor_inside| MouseEvent::Up { cursor_inside }),
    ]
}

/// Strategy to generate a random sequence of mouse events (1 to 30 events).
fn arb_mouse_event_sequence() -> impl Strategy<Value = Vec<MouseEvent>> {
    proptest::collection::vec(arb_mouse_event(), 1..30)
}

/// Compute the expected ButtonState after applying a mouse event.
fn expected_state_after(event: MouseEvent) -> ButtonState {
    match event {
        MouseEvent::Enter => ButtonState::Hover,
        MouseEvent::Exit => ButtonState::Idle,
        MouseEvent::Down => ButtonState::Pressed,
        MouseEvent::Up {
            cursor_inside: true,
        } => ButtonState::Hover,
        MouseEvent::Up {
            cursor_inside: false,
        } => ButtonState::Idle,
    }
}

/// Apply a mouse event to the button.
fn apply_event(btn: &mut NSButton, event: MouseEvent) {
    match event {
        MouseEvent::Enter => btn.handle_mouse_entered(),
        MouseEvent::Exit => btn.handle_mouse_exited(),
        MouseEvent::Down => btn.handle_mouse_down(),
        MouseEvent::Up { cursor_inside } => btn.handle_mouse_up(cursor_inside),
    }
}

// Feature: macos-visual-polish, Property 2: Button state machine transitions produce correct colors
//
// **Validates: Requirements 4.5, 4.6, 6.1, 6.2, 6.3, 6.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_button_state_machine_transitions_produce_correct_colors(
        events in arb_mouse_event_sequence()
    ) {
        let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));

        prop_assert_eq!(btn.interaction_state(), ButtonState::Idle);
        prop_assert_eq!(btn.current_background_color(), ButtonState::Idle.background_color());

        for (i, event) in events.iter().enumerate() {
            apply_event(&mut btn, *event);

            let expected_state = expected_state_after(*event);
            let actual_state = btn.interaction_state();
            let actual_color = btn.current_background_color();
            let expected_color = expected_state.background_color();

            prop_assert_eq!(
                actual_state, expected_state,
                "After event {:?} (step {}): expected state {:?}, got {:?}",
                event, i, expected_state, actual_state
            );

            prop_assert_eq!(
                actual_color, expected_color,
                "After event {:?} (step {}): expected color {:?} for state {:?}, got {:?}",
                event, i, expected_color, expected_state, actual_color
            );
        }
    }
}

// Feature: macos-visual-polish, Property 3: All buttons use theme-defined corner radius
//
// **Validates: Requirements 6.5**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_all_buttons_use_theme_defined_corner_radius(
        radius in 0.0f32..50.0f32
    ) {
        {
            let reg = crate::theme::registry::get_registry();
            let mut guard = reg.lock().unwrap();
            let variant = guard.current_variant().to_string();
            let mode = guard.current_mode().to_string();
            let material = guard.current_material().to_string();
            let mut theme = guard.compose(&variant, &mode, &material);
            theme.layout.geometry.insert(
                crate::theme::keys::RADIUS_BUTTON.to_string(),
                radius,
            );
            guard.register("__test_prop3", theme);
            guard.set_current("__test_prop3");
        }

        let effective_radius = crate::theme::f32_or(crate::theme::keys::RADIUS_BUTTON, 6.0);
        prop_assert!(
            (effective_radius - radius).abs() < f32::EPSILON,
            "Expected button corner radius {}, got {} from theme",
            radius, effective_radius
        );

        let mut btn = NSButton::new(NSRect::sized(100.0, 30.0));
        btn.view_mut().set_radius_token(crate::theme::keys::RADIUS_BUTTON);
        btn.view_mut().apply_theme();
        let view_radius = btn.view_mut().corner_radius();
        prop_assert!(
            (view_radius - radius).abs() < f32::EPSILON,
            "Expected button view corner radius {}, got {} after apply_theme()",
            radius, view_radius
        );
    }
}
