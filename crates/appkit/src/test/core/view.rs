use super::*;

/// A root at 10, 10 holding one child at 5, 5 of its own coordinates.
fn nested(seen: &Seen, child_handles: bool, root_handles: bool) -> Box<Tile> {
    let child = Tile::new("child", PRect::new(5.0, 5.0, 20.0, 20.0), child_handles, seen, Vec::new());
    Tile::new("root", PRect::new(10.0, 10.0, 100.0, 100.0), root_handles, seen, vec![child])
}

#[test]
fn a_view_is_drawn_before_what_it_holds_each_where_it_is_in_the_window() {
    let seen = Seen::default();
    let root = nested(&seen, false, false);
    let mut board = Board::default();
    show(root.as_ref(), &mut board, PPoint::default());
    let rect: Vec<PRect> = board.fill.iter().map(|(rect, _)| *rect).collect();
    assert_eq!(rect, [PRect::new(10.0, 10.0, 100.0, 100.0), PRect::new(15.0, 15.0, 20.0, 20.0)]);
    // Shown inside something else, everything moves with it.
    let mut board = Board::default();
    show(root.as_ref(), &mut board, PPoint::new(100.0, 0.0));
    assert_eq!(board.fill[1].0, PRect::new(115.0, 15.0, 20.0, 20.0));
}

#[test]
fn an_event_goes_to_the_innermost_view_under_the_pointer_in_its_own_coordinates() {
    let seen = Seen::default();
    let mut root = nested(&seen, true, true);
    assert!(send(root.as_mut(), &PEvent::PointerDown(PPoint::new(20.0, 20.0))));
    assert_eq!(*seen.borrow(), [("child", PEvent::PointerDown(PPoint::new(5.0, 5.0)))]);
}

#[test]
fn what_a_view_does_not_handle_is_passed_up_to_the_view_that_holds_it() {
    let seen = Seen::default();
    let mut root = nested(&seen, false, true);
    assert!(send(root.as_mut(), &PEvent::PointerDown(PPoint::new(20.0, 20.0))));
    assert_eq!(*seen.borrow(), [("child", PEvent::PointerDown(PPoint::new(5.0, 5.0))), ("root", PEvent::PointerDown(PPoint::new(10.0, 10.0)))]);
    // And when nobody handles it, everybody on the way was asked.
    let seen = Seen::default();
    let mut root = nested(&seen, false, false);
    assert!(!send(root.as_mut(), &PEvent::PointerDown(PPoint::new(20.0, 20.0))));
    assert_eq!(asked(&seen), ["child", "root"]);
}

#[test]
fn a_view_the_pointer_is_not_over_is_not_asked() {
    let seen = Seen::default();
    let mut root = nested(&seen, true, true);
    assert!(!send(root.as_mut(), &PEvent::PointerDown(PPoint::new(200.0, 200.0))));
    assert!(seen.borrow().is_empty());
    // Over the root and beside the child: the root alone.
    assert!(send(root.as_mut(), &PEvent::PointerDown(PPoint::new(90.0, 90.0))));
    assert_eq!(asked(&seen), ["root"]);
}

#[test]
fn of_two_views_under_the_pointer_the_one_drawn_last_is_asked_first() {
    let seen = Seen::default();
    let under = Tile::new("under", PRect::new(0.0, 0.0, 50.0, 50.0), true, &seen, Vec::new());
    let over = Tile::new("over", PRect::new(25.0, 25.0, 50.0, 50.0), true, &seen, Vec::new());
    let mut root = Tile::new("root", PRect::new(0.0, 0.0, 100.0, 100.0), true, &seen, vec![under, over]);
    assert!(send(root.as_mut(), &PEvent::PointerDown(PPoint::new(30.0, 30.0))));
    assert_eq!(asked(&seen), ["over"]);
    // Where only the lower one is, it is asked.
    seen.borrow_mut().clear();
    assert!(send(root.as_mut(), &PEvent::PointerDown(PPoint::new(10.0, 10.0))));
    assert_eq!(asked(&seen), ["under"]);
}

#[test]
fn a_key_goes_to_the_view_itself() {
    let seen = Seen::default();
    let mut root = nested(&seen, true, true);
    assert!(send(root.as_mut(), &PEvent::KeyDown('a')));
    assert_eq!(*seen.borrow(), [("root", PEvent::KeyDown('a'))]);
}

#[test]
fn a_view_can_be_put_somewhere_else() {
    let seen = Seen::default();
    let mut root = nested(&seen, true, true);
    root.set_frame(PRect::new(500.0, 500.0, 10.0, 10.0));
    assert_eq!(root.frame(), PRect::new(500.0, 500.0, 10.0, 10.0));
    assert!(!send(root.as_mut(), &PEvent::PointerDown(PPoint::new(20.0, 20.0))));
}
