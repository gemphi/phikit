//! Timeline Panel Builders

use crate::svg_icons::{self, IconId};
use crate::theme;
use crate::timeline::NSTimelineView;
use crate::{NSAutoresizingMask, NSButton, NSColor, NSRect, NSTextAlignment, NSTextField, NSView};

/// Modular helper to construct keyframe timeline control chrome
pub fn build_rcp_timeline(view: &mut NSView, rect: NSRect, timeline_view: &mut NSTimelineView) {
    let toolbar_height = 30.0;
    let mut toolbar = NSView::new(NSRect::sized(rect.width(), toolbar_height))
        .with_name("TimelineToolbar");
    toolbar.set_background_color(Some(theme::color(theme::CLR_BG_CONTROL)));
    toolbar.set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::MAX_Y_MARGIN,
    );

    let timeline_rect = NSRect::new(
        0.0,
        toolbar_height,
        rect.width(),
        (rect.height() - toolbar_height).max(0.0),
    );
    timeline_view.view_mut().set_frame(timeline_rect);
    timeline_view.view_mut().set_name("TimelineTracks");
    timeline_view.view_mut().set_autoresizing_mask(
        NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
    );

    let btn_y = 4.0;
    let btn_h = 22.0;
    let btn_gap = 4.0;

    let play_w = 32.0;
    let mut play_btn = NSButton::new(NSRect::new(8.0, btn_y, play_w, btn_h));
    play_btn.set_image(svg_icons::get(IconId::Play));
    play_btn.view_mut().set_name("PlayButton");
    let timeline_play = timeline_view.clone();
    play_btn.set_action(move || {
        timeline_play.play();
    });
    toolbar.add_subview(play_btn.view_clone());

    let pause_x = 8.0 + play_w + btn_gap;
    let pause_w = 32.0;
    let mut pause_btn = NSButton::new(NSRect::new(pause_x, btn_y, pause_w, btn_h));
    pause_btn.set_image(svg_icons::get(IconId::Pause));
    pause_btn.view_mut().set_name("PauseButton");
    let timeline_pause = timeline_view.clone();
    pause_btn.set_action(move || {
        timeline_pause.pause();
    });
    toolbar.add_subview(pause_btn.view_clone());

    let stop_x = pause_x + pause_w + btn_gap;
    let stop_w = 32.0;
    let mut stop_btn = NSButton::new(NSRect::new(stop_x, btn_y, stop_w, btn_h));
    stop_btn.set_image(svg_icons::get(IconId::Stop));
    stop_btn.view_mut().set_name("StopButton");
    let timeline_stop = timeline_view.clone();
    stop_btn.set_action(move || {
        timeline_stop.stop();
    });
    toolbar.add_subview(stop_btn.view_clone());

    let time_label_width = 80.0;
    let time_label_margin = 8.0;
    let mut time_label = NSTextField::new(NSRect::new(
        (rect.width() - time_label_width - time_label_margin).max(120.0),
        btn_y + 2.0,
        time_label_width,
        18.0,
    ));
    time_label.set_string_value("0.00s");
    time_label.set_text_alignment(NSTextAlignment::Right);
    time_label.view().set_name("TimeLabel");
    time_label
        .view_mut()
        .set_autoresizing_mask(NSAutoresizingMask::MIN_X_MARGIN | NSAutoresizingMask::MAX_Y_MARGIN);
    toolbar.add_subview(time_label.view_clone());

    view.add_subview(toolbar);
    view.add_subview(timeline_view.view().clone());
}
