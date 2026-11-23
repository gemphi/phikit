use crate::pui::core::*;

#[test]
fn a_pointer_event_has_a_place_and_a_key_has_none() {
    let at = PPoint::new(3.0, 4.0);
    for event in [PEvent::PointerDown(at), PEvent::PointerUp(at), PEvent::PointerMove(at), PEvent::Scroll(at, PPoint::new(0.0, 1.0))] {
        assert_eq!(event.at(), Some(at), "{event:?}");
    }
    assert_eq!((PEvent::KeyDown('a').at(), PEvent::KeyUp('a').at()), (None, None));
}

#[test]
fn a_view_sees_an_event_in_its_own_coordinates() {
    let origin = PPoint::new(10.0, 20.0);
    assert_eq!(PEvent::PointerDown(PPoint::new(13.0, 24.0)).seen_from(origin), PEvent::PointerDown(PPoint::new(3.0, 4.0)));
    assert_eq!(PEvent::PointerUp(PPoint::new(10.0, 20.0)).seen_from(origin), PEvent::PointerUp(PPoint::default()));
    assert_eq!(PEvent::PointerMove(PPoint::new(0.0, 0.0)).seen_from(origin), PEvent::PointerMove(PPoint::new(-10.0, -20.0)));
    // How far the wheel turned is not a place, and stays.
    let turned = PPoint::new(0.0, 3.0);
    assert_eq!(PEvent::Scroll(PPoint::new(13.0, 24.0), turned).seen_from(origin), PEvent::Scroll(PPoint::new(3.0, 4.0), turned));
    assert_eq!(PEvent::KeyDown('x').seen_from(origin), PEvent::KeyDown('x'));
}
