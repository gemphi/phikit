use super::Board;
use crate::pui::core::*;

#[test]
fn an_icon_is_drawn_by_its_name_at_its_size_where_it_is_put() {
    let icon = PIcon::new("send", PSize { width: 16.0, height: 16.0 });
    let mut board = Board::default();
    icon.draw(&mut board, PPoint::new(100.0, 40.0));
    assert_eq!(board.icon, [("send".to_string(), PRect::new(100.0, 40.0, 16.0, 16.0))]);
    // It is a mark and nothing else: no fill, no text.
    assert!(board.fill.is_empty() && board.text.is_empty());
}
