# appkit - specs

Each piece: what it must do, what Apple calls the like of it, what was
read before it was written, and where it is tested. Tests are under
`src/test`, in the folder of the code; Phigen's are under
`app/phigen/src/test`.

## What a thing on screen is

| Piece | Must | Apple | Read | Test |
|---|---|---|---|---|
| `PDraw`, `PView` | draw itself; have a rectangle, hold views, answer an event from the innermost view up; take in what arrived once a frame | `NSResponder`, `NSView` | `old/core/view`, `old/core/responder` | `core/view.rs` |
| `PEvent` | carry a press, a move, the wheel, a key; keys that write nothing are characters of Apple's function-key range | `NSEvent`, `NSUpArrowFunctionKey` | `old/core/event`; gpui's `interactive.rs` | `core/event.rs`, `screen/key.rs` |
| `PPaint`, `PSheet` | fill, round, blend one colour into another, lay glass, set text, draw an icon, clip, measure; keep what was drawn as marks a screen plays | `NSGraphicsContext`, Core Graphics | `old/core/graphics`; gpui's `scene.rs`, `examples/painting.rs` | `core/paint.rs`, `core/sheet.rs` |
| `PControl` | hold a value; send an action when a hand changes it, not when the app stores one | `NSControl` | `old/controls/control` | `control/mod.rs` |
| `PButton` | count presses; draw its label in the middle of its height | `NSButton` | `old/controls/button` | `control/button.rs` |
| `PToggle` | be on or off | `NSSwitch` | `old/controls/switch` | `control/toggle.rs` |
| `PField` | take text at a caret; arrows, Home, End move it; Backspace and Delete take a letter | `NSTextField` | `old/controls/text_field`; gpui's `examples/input.rs` | `control/field.rs` |
| `PScroll` | move a content longer than its room, no further than its ends | `NSScrollView` | `old/views/scroll_view` | `layout/scroll.rs` |
| `wrap` | break text between words into lines that fit | Core Text line breaking | gpui's `text_system/line_wrapper.rs` | `layout/wrap.rs` |
| `PFit`, `PPaint::picture`, `PMark::Picture`, the screen's `Album` | draw a picture found among the app's assets by its name, in its own colours, its corners from the caller: whole and never larger than it is, or covering its rectangle, or one cell of a sheet; decode its file once, off the window's thread where there are threads; draw nothing for a name that is no picture | `NSImageView`, `NSImage`, `CALayer.contentsGravity` | gpui's `window.rs` (`paint_image`), `assets.rs` (`RenderImage`), `platform.rs` (`to_image_data`) | `core/paint.rs`, `core/sheet.rs`, `screen/play.rs` |
| `PShape::Whole` | lay a modal over the whole window, for what is looked at alone | `NSWindow.toggleFullScreen`, a Quick Look panel | `pui/window/panel.rs` | `window/panel.rs` |
| a secret field, a worn look, a chosen file | draw a mark for each character of a field that is secret, and keep its value; wear colours and sizes over the look shown, whichever look is turned to; ask the person for a file in the system's dialog and tell the app a word with its path; take a word for the app from outside the window | `NSSecureTextField`, `NSAppearance`, `NSOpenPanel` | `pui/control/field.rs`, `src/theme/shade.rs`, `src/screen/errand.rs`, `pui/window/stage/laid.rs` | `control/field.rs`, `theme/shade.rs`, `window/stage/bar.rs` |
| a bare modal | leave a modal's panel without a head where its table says `bare`: no title, neither button, all of it the content's; Escape closes it still | a borderless `NSPanel` | `pui/window/modal.rs`, `pui/window/panel.rs` | `window/panel.rs` |
| errands: `PDeed::Copy`, `Launch`, `Save` | let a part wish for what only the system can do; keep it on the stage until the screen takes it; copy a text, open a file in the system's own program, ask in the system's dialog where a copy is kept and keep one, with the file's ending when the dialog gave none | `NSPasteboard`, `NSWorkspace.open`, `NSSavePanel` | gpui's `write_to_clipboard`, `open_with_system`, `prompt_for_new_path`; `src/screen/errand.rs` | `window/stage/bar.rs`, `screen/play.rs` |
| `PPaint::natural`, `PSheet::sized` | say how large a picture is, once the screen has read it, so a view can lay it out larger than its room or know where it lies | `NSImage.size` | `src/screen/album.rs` | `core/sheet.rs` |
| the pointer's moves | send a part where the pointer is, and draw the window again only for a part that took notice | `mouseMoved`, tracking areas | gpui's `on_mouse_move`; `src/screen/mod.rs` | Phiogy's `viewer::what_a_control_is_called_...` |
| `PFilm`, `PStill`, `PAsset::film`, the album's `stale` | draw a picture that moves by the same mark as one in a file: an app shows a frame under a name from any thread, the latest is what is drawn, another frame follows while any picture moves, and the frame before is let go of, since the renderer keeps what it was given until told | `AVSampleBufferDisplayLayer`, `CALayer.contents` | Zed's `livekit_client` (`remote_video_track_view.rs`), `src/screen/album.rs` | `core/film.rs`, `screen/play.rs` |
| `figure` (`size`, `end`, `start`) | draw a number a person reads on a ground of its own, the same in every app: a label's light as high as the theme says, with room beside the text and corners as round as the theme says, never narrower than high; against the ground it stands on where it is asked to; bare where the theme gives it no height, and nothing for no text | `UILabel` in a capsule, a badge's `UIBackgroundConfiguration` | `puikit/packages/pui-components` (`Badge`, the wallet rows of `phi-kit/src/AccountDrawer.tsx`) | `core::figure::a_figure_stands_on_a_ground_of_its_own_with_its_text_in_the_middle`, `..._runs_against_it_and_one_digit_stands_on_a_round_one`, `..._is_bare_where_the_theme_gives_it_no_ground_and_nothing_with_no_text` |
| `PCaption` (the mark is the pin), `PRail`, `PItem::face`, `PView::hand`, `PStage::hand`, the ground of a part | the app's mark over the bar on a ground as large as every icon's, the pin in its place under the pointer, a press putting the menu away and bringing it back; rows an app adds to its bar while it runs, each with a picture, as many as fit, asked again when the window is laid out and once a frame; the row of the bar or of a menu under the pointer lit; a hand wherever a press does something; and every part of the body on a ground of its own where the theme gives `part.inset` | `NSToolbar` and a sidebar's toggle, `NSCursor.pointingHand`, `UIHoverGestureRecognizer`, a Discord-like rail of servers | `puikit/src/components/navigation/AppShell` (`Brand.tsx`, the rail) | `window::part::caption::*`, `window::part::activity::draw::*`, `window::stage::bar::under_a_caption_...`, Phiogy's `flow::step_9_...`, `flow::step_44_...` |
| `PIcon`, `PAsset` | name a drawing; find a file by its path | `NSImage` symbols, `NSBundle` | gpui's `assets.rs`, `svg_renderer.rs` | `core/icon.rs`, `core/asset.rs` |
| `PMode` | say desk, hand or space; a window narrower than `mode.tsv` gives is a hand's | size classes, `UIUserInterfaceIdiom` | - | `core/mode.rs` |

## How a window is built

| Piece | Must | Apple | Read | Test |
|---|---|---|---|---|
| `PApp` | supply four sets, a theme, the bar's rows, a user, the view of each part; say what it names and has not supplied | `NSApplication`, its delegate | `old/core/application`; Phiber's `app` | `app.rs` |
| `PSet`, `PEntry`, `PLevel` | read a level's entries from a table; keep the first of a name | - | Phiber's `level.rs`, `spec.rs` | `window/set.rs`, `window/mod.rs` |
| `PWindow` | show one space; keep the area in front, the modal open, the row of the bar picked and its menu put away or drawn out | `NSWindow`, `NSWindowController` | `old/core/window`, `old/workspace` | `window/mod.rs`, `window/deed.rs` |
| `PWindow::place` | put the areas side by side at a desk by their zone's share; one in front in a hand; an arc in space | `NSSplitViewController`, `UINavigationController` | Phiber's `zone`, `area` | `window/place.rs` |
| `PHost::lay` | stack regions, fixed ones at the theme's height; side panes hug their edges | `NSStackView`, a title bar accessory | Phiber's `host`, `region`, `pane` | `window/host/lay.rs` |
| `PStage` | lay out, draw each part inside its own rectangle, send events, do deeds and wishes | `NSWindowController` with its content view controller | gpui's `window.rs` dispatch | `window/stage.rs` |
| `PFrame` | give the caption, the bar, its menu, the tabs and the body their rectangles: the caption the row at the top, in a theme that has one, and everything else under it; the bar and its menu a margin clear of the window's edges; the bar at the bottom in a hand, its menu over the body | `NSSplitView` sidebar, `UITabBar` | - | `window/frame.rs` |
| `PCaption`, `PGrip` | the row at the window's top, where a system has its title bar: the app's mark over the bar, doing what the app's table names; the app's name while a menu stands beside the bar; the pin at the end of the cell over the bar and its menu, which puts the menu away and brings it back (draws it out, in a hand); in a window with no bar of the system's, the window's three buttons at the right end and, between everything pressed, where the window is moved by; a modal's panel is laid in the room under it | a window's title bar with `titlebarAppearsTransparent`, a toolbar's sidebar toggle | `puikit/src/components/navigation/AppShell/Brand.tsx`; Zed's `platform_title_bar` | `window/part/caption.rs`, `src/screen/mod.rs` |
| `PActivity` | a column of icons: the app's mark at its head, where the theme has no caption, its rows at the top, the middle and the foot, the user last; a press on a row does its own deed, if it has one, and shows its menu beside the bar, and again puts the menu away; lie as squares, the menu drawn out over the space | a sidebar (`NSOutlineView` source list); puikit's `AppShell` rail | `old/views/sidebar`, `old/views/activity_bar`; `puikit/src/components/navigation/AppShell` | `window/part/activity/` |
| `PItem`, `PGroup`, `PSeat`, `menu` | a row's menu in groups seated at the top, the middle and the foot; each group on a ground of its own, the row shown lit against it; one group may hold a host of the app's and takes the room the others leave | `NSOutlineView` groups; puikit's `AppShell` panel | `puikit/src/components/navigation/AppShell/Menu.tsx` | `window/part/menu.rs` |
| `PTab` | a tab for each space, the one shown filled; narrow to share the bar | `NSTabView`, `NSSegmentedControl` | `old/views/tab_view` | `window/part/tab.rs` |
| `PShape`, `PModal` | a modal's shape, from the app's table: a square in the middle, a drawer at the right edge, a sheet at the foot | `NSPanel`, an inspector, `UISheetPresentationController` | puikit's `overlays/Drawer` (right, bottom) | `window/modal.rs` |
| `PPanel` | a drawer as wide as the theme says down the right edge, and a sheet a share of the window's height at the foot, each a margin clear of the window; a modal as a square in the middle that fills as much of the window's space as the theme says (half), and no more of its shorter side than a share, another in a hand; expand to the window and back; close by its button, Escape, a press beside it | a sheet, `NSPanel` | `old/controls/alert` | `window/panel.rs` |
| `PProfile` | show who the user is from a table, what is true of them now (asked of the app once a frame), and a pill for each look; turn the window to the look pressed and tell the app; scroll when longer than its panel | an account view | - | `window/part/profile.rs` |
| `PDeed`, `PWish` | name what a press on the furniture does, an area coming to hold another host among them; let a part ask for one | target and action, the responder chain | `old/core/responder` | `window/deed.rs` |

## Beside the kit

| Piece | Must | Apple | Read | Test |
|---|---|---|---|---|
| `PTheme`, `PLook` | give a colour, a size or a word for a token; read them from a table, the last row winning | `NSAppearance`, asset catalogs | `old/theme`; Phiber's `theme` | `theme/mod.rs`, `theme/look.rs` |
| `PShade` | hold several looks and show one; turn to another while the window is open, so the next frame is drawn in it | `NSAppearance`, light and dark (`effectiveAppearance`) | Phiber's `theme` | `theme/shade.rs` |
| `PLight`, `light` | light a surface from its foot, a label from its left or its right, at one of three strengths of the theme's one light; a pane is glass or the window's ground under that light, and one at an edge of the window is lit toward the page; fill flat in a theme with no light | puikit's `--phi-surface`, `--phi-label` and `--phi-label-end` | `puikit/src/tokens/tokens.scss`; `AppShell/styles.module.scss` (`.nav`) | `core/light.rs` |
| `PGlass` | say what a pane of glass does to what is behind it, from the theme; none where the theme gives no blur | `NSGlassEffectView`, `NSVisualEffectView` | `gpu/gpui_glass`; Phiber's `theme/style/glass.rs` | `core/glass.rs` |
| `PTable` | names and what they stand for, from a file | `Localizable.strings` | - | `service/table.rs` |
| `PFolder`, `PBundle` | find an asset on disk, or built into the program; the app's own before the shared | `NSBundle` | gpui's `assets.rs`; Phiber's `web_assets` | `service/folder.rs`, `service/bundle.rs` |
| `PStore` | read and write a person's own files for an app | `FileManager`, Application Support | - | `service/store.rs` |
| `PCall`, `PStream` | post to a server, or only ask it, and give its answer a line at a time; stop when asked | `URLSession` with a bytes stream | Ollama's chat API | `service/call.rs` |
| `screen` | open a stage in a window; play marks; turn keys, presses and the wheel into events; keep the app alive in a browser | `NSApplicationMain`, Core Animation's commit | gpui's `examples/hello_world.rs`, `painting.rs`, `input.rs`; Phiber's `app/launch.rs` | `screen/key.rs`, `screen/play.rs`; the window itself by opening Phigen |

## Not met

| Must | Where it stands |
|---|---|
| no `use gpui` outside `src/screen` | met; but gpui's interface half is not yet transformed into `pui` |
| one GPU layer | not met: DirectX at a desk on Windows, wgpu in a browser |
| glass on an appkit window | met on DirectX: a look that is glass has the window show the desktop blurred, and a pane (the bar, a modal's panel) is `PMark::Glass`, drawn from `gpu/gpui_glass`'s plan. Elsewhere a pane shows its tint alone |
| animation | not here: phiphx owns the curves, phiecs is to play them |
| a headset | placed (`PWindow::place`), shown by nothing |
