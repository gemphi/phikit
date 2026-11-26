# The asset library: what it holds and what it can do

Taken 27 November 2026, from the files on disk and the code on `phiapp`,
before the couches, saving, the web's 3D and folder sync are built. What
is asked: 32 couches; turn one, press Save, get a USD that plays on Apple
computers; the web shows 3D where an asset has it; folders synced with
one on our sites; calls from the web answered on the desktop.

## What it holds

| Place | Files | Kind |
|---|---|---|
| `asset/once/` (Once City) | 115 `.usda`, 31 MB | written by phiphx's `thing_usd` from `data/once/*.tsv` |
| `asset/shape/` (Shapes) | 13 `.usda` | cube, cone, capsule, dice and the like |
| Phiegg's collections and clubs | pictures | through `shelf.json` (puikit `app/phiogy/library.py`) |
| A folder of this disk | anything | a source that is not USD is turned into USD beside it (`data/convert.tsv`) |

Once City, by kind: 16 trees, 16 lands, 13 couches, 10 jetskis, 8
zebras, 8 phijets, 4 rockets, 4 flycars, 3 seats, 3 pads, 3 lotus, 3
lioness, 3 jets, and one each of the rest (lion, gate, lamp, dome
pieces, ship, glider).

## The couches: 16 then, 32 now

Since this was taken: sixteen scroll couches from the drawing below
(`couch-scroll-1` to `-16`, `script/couch.py` from `data/once/couch.tsv`),
eight steel-band lounge chairs and settees after the photo of the same
day (`lounge-*`), and their table with two glasses (`table-tulip`).
What stood then:

| Have | Made from |
|---|---|
| 13 couches | a letter of Phi-Pill Bold as both side panels: L h n u d b r e c z J w, and the bird as the rose chaise |
| ottoman, pouf, stool | the I lying down, the o, the m |

16 seats in all, one `.usda` each, every one from `data/once/thing.tsv`
(side panel, body `seat-1/2/3`, its frame, cushion and pillow colours).
**16 more are needed for 32.** Nothing else on disk is a couch: the
gemkit `GemEntityCouch` (phiadk-js) is an empty stub, and the only
drawing is `IMG_20260930_couCHES.jpg` on the Desktop: one frame, mirrored,
its arms rolled into scrolls, its back a lattice of openings, small to
large to small.

## Turning and saving

| Can | Cannot yet |
|---|---|
| Open an item in 3D (`part/figure.rs`): drag turns it, the wheel brings it nearer, four views, wire | save what is shown: no Save, no export of any item |
| Read and write `.usda` | write `.usdz`, the one file Apple's Quick Look plays on a Mac, iPhone or Vision Pro |

What Save needs: the item's USD with the turn the person gave it as its
root's transform and its looks as set, packed as `.usdz` (a zip, stored
not compressed, each file at a 64-byte boundary), into the folder the
person picks. One writer in the kit, so every app with a model has it.

## The web

| Web | 3D today |
|---|---|
| `phiegg.com/phiogy/` (puikit, React) | none: no viewer, no model on any item page |
| This program built for the browser (`web/`, WebGPU) | built, never opened; it carries the same figure as the desktop |

What it needs: an item that has a model names its `.usdz` key, and its
page shows it turned by a drag, with a link that opens the file in
Quick Look on Apple devices.

## Folders synced with the site

Planned in `SHARED-LIBRARY.md` section 4; most of it is not built.

| Built | Not built |
|---|---|
| `phiegg.com/cdn/` serves `/srv/phi-cdn` | `POST /file`: upload by hash to `/srv/phi-cdn/lib/<sha256>.<ext>` |
| Phiora's change log, `GET /tree/change` | `after=<seq>`, so a window asks only what is new |
| A far picture is read once and kept in a cache | the `asset` record (name, key, size, folder, who, draft → review → approved) |
| What the window makes reaches others through Phiora (`da27d71bd`) | the library's **Team** place, and a local folder kept equal to it both ways |

## Calls, web to desktop

Calls go through `phiegg.com/wrt`, and the desktop Phiogy is on
phiegg.com by default. Between two different networks they need the
relay: the AWS security group must let in UDP+TCP 3478 and UDP
49152-49407, which only you can open.

## Order

1. ~~Sixteen more couches, from tables, as the first sixteen are.~~ Done.
2. Save: the turned item as `.usdz`, a kit writer.
3. Folder sync: `POST /file`, the `asset` record, `after=`, the Team place.
4. The web shows an asset's `.usdz` in 3D and opens it in Quick Look.
5. Calls: web callers answered on the desktop, with sound, end to end.
