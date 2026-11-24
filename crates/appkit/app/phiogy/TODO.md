# Phiogy - to do

What is left, the nearest first. What is done is in [`PLAN.md`](PLAN.md)
and, step by step, in [`FLOW.md`](FLOW.md).

## Not committed

Nothing of this is committed. It is this folder, one line of the
workspace's `Cargo.toml` (the member). Git keeps neither the
workspace's `Cargo.lock`, to which cargo added `phiogy` and, in round
two, `libwebrtc` with the crates it needs, nor the three
scripts under `target/` (`phiogy_run.ps1`, `phiogy_shot.ps1`,
`phiogy_doc.ps1`), nor the three added with the call's pictures
(`phiogy_snap.ps1`: a picture of a window already running;
`phiogy_press.ps1`: a press at a place in it; `phiogy_peak.ps1`: the
system's meter of its sound out). `script/scene.mjs`, in this folder, stages every screen of a
call and keeps its picture in `doc/scene/`. For the microphone, the sound out
and the camera cargo added `cpal` 0.15.3 and `windows` 0.54 to the
lock, with the crates they need, all from its cache. One entry was added to the workspace folder's
`.claude/launch.json`: `phiogy-line`, the web Phiogy pointed at this
line on port 4311.

## Not in this round

These were left out of round one on purpose. The first was done in
round two; none of the others is begun.

- [x] **This window as a caller, with WebRTC (round two).** It joins a
  room's call over the line and, built with `rtc`, sends and takes
  sound and picture; proven against a Chrome page both ways
  ([`FLOW.md`](FLOW.md)). What is left of it is under "The caller".
- [ ] **A relay (TURN).** With no ICE server a call reaches the same
  machine and the same network. `data/ice.tsv` is where a server goes.
  One is run on the server in the cloud (coturn; puikit:
  `deploy/relay`), and the line there hands it to every page
  (`PHI_CALL_ICE`). It is not reached from outside until the server's
  firewall lets in UDP 3478, TCP 3478 and UDP 49152-49407. Its word is
  one for everybody, handed to guests too: a word made for each caller
  and for a short time is not built.
- [ ] **A forwarding unit.** A call is a mesh of 8. Rooms above that
  need a unit that forwards.
- [ ] **Messages reaching the other person.** The board holds calls
  only; a room shows who is on its line and its people, not what was
  said.
- [ ] **Taking the route over on the server in the cloud.** On this
  machine it is taken over: Phiora started with `PHI_CALL_LINE` and
  `PHI_CALL_KEY` keeps no rooms and passes `/call` to this line (its
  `test/call-line.test.mjs`, and a whole call between two Chrome pages
  with sound, both cameras and a shared screen). On the server it is
  taken over too (22 November 2026): `Dockerfile` builds `phiogy --line`
  for Linux from what `script/ship.sh` sends, and it runs as the
  stack's service `phiogy` beside Phiora (puikit:
  `app/docker-compose.yml`, `app/DEPLOY.md`); two test pages found
  each other on `https://phiegg.com/api/call/...`, and the web Phiogy,
  published at `https://phiegg.com/phiogy/`, carried a whole call
  between two Chrome pages through it (three runs of four passed every
  step; in one the first page still read "is calling" after both had
  hung up, which is not explained). What is left: the server's Docker
  has no BuildKit, so each build is a whole one (about five minutes).

## As the web Phiogy: what is built and what is not

Asked on 22 November 2026: the native Phiogy is to be a replica of the
web one, its configs, its windows and the data Phiora serves for it.

Built:

- [x] the line answers `/app/KEY`, `/me` and `/state/NAME` as Phiora does
- [x] the bar's menu is the web's `nav`, with its headings
- [x] a page is its blocks; one added to the app's file opens
- [x] a room shows what was said in it, with its people
- [x] a picture mark in appkit (`PPaint::picture`, `PFit`), and a modal
  that fills the window (`whole`)
- [x] faces, places' marks and posted pictures, from the web's own files
- [x] the viewer of a posted picture
- [x] the room's tools: writing a message, kept in the person's folder
  and on Phiora; answering, liking and pinning; marks given and taken
  back; search in the room; the pinned listed; a person beside the
  room, their pictures in the viewer
- [x] posting a picture of this disk; muting a person; a theme's
  swatch, and wearing one that is bought
- [x] in the account: one heading; a handle and a password; the
  sessions and signing the others out; a copy of one's data
- [x] share and open in the viewer pressed in the real window: the
  clipboard held the file's path after share (and was put back); the
  system's Photos opened the file, and that window was closed
- [x] the hosts with their marks: joining, leaving, making one from a
  template and removing it
- [x] direct conversations with anybody, and friends, each a page
- [x] the shops, a thing's page, the bag, and paying from the wallet
  (play money; Phiora is asked to move it where it answers)
- [x] the account: who the person is, the wallet, the look, and being
  one of Phiora's demo members
- [x] the viewer as the web's: no head of a panel's, one row of who
  and when, the tools and the way out; larger and whole again; share,
  save and open through the system (appkit's errands); what a control
  is called under the pointer
- [x] one source at run time: Phiora is asked for the app's document
  first, and the app's file is read where it does not answer
  (`src/phiora.rs`, `data/phiora.tsv`; `node script/phiora.mjs`
  checks it against the real Node Phiora)

Not built, the nearest first:

- [ ] in the viewer: a web address to share (the window knows none the
  picture is served at; it copies where the file is); moving a strip
  longer than the window by hand; the strip named to what reads a
  window aloud (`view.strip` waits for that)
- [ ] a face on who is on the line (a caller is a guest there), and in
  the profile
- [ ] a sign-in Phiora takes: the form, the masked password and what
  Phiora refuses are built and seen, and a sign-in it takes is tested
  against something that answers as Phiora does. Against the real
  Phiora it was tried only as far as a refusal: the seed's members
  have no passwords, and making an account to try one is the user's
  to say
- [ ] a message reaching the other people of a room: as on the web,
  what a person writes is their own document; nothing carries it to
  anybody else
- [ ] in a direct room the other person's profile beside it from the
  start, as the web has it (here who is on the line stands there
  until a face is pressed)
- [ ] a picture posted from this disk is named by where the file is,
  so the web, which cannot open a file of a disk, does not show it;
  Phiora has nowhere to put a file yet
- [ ] a Phiora behind HTTPS: it is asked over plain HTTP
- [ ] the rooms' and the hosts' files from Phiora: the web reads them
  from the app's own files too, so there is nothing to ask yet
- [ ] in the shops: offers and selling (`shop.ts` has them; no page of
  the web kit uses them); reading Phiora's answer to a payment back
  at once (the wallet is read from Phiora when the window opens)
- [ ] in the account: making an account, an email, changing a
  password, deleting (the web's `AccountDialog`); the colour, the
  font, a mode that follows the system. Deleting is left out on
  purpose: nothing here deletes anything of Phiora's
- [ ] what a worn theme changes is my reading (`data/wear.tsv`: the
  light takes its own colour, the field's line its second); the web
  changes its palette, which this window does not have
- [ ] a voice room of a host the person made: its line works as any
  room's; who is in it is not listed under it in the rail
- [ ] on the line: the wallet and what is owned (Phiora writes those),
  the shop, accounts, the records of apps (`app`, `menu`, `page`,
  `block`) from which Phiora composes an app's document: the line
  reads the app's file, which is what Phiora falls back to
- [ ] a person's documents kept on disk: they last as long as the line

## The native peer: what it is built on

**`libwebrtc`**, LiveKit's safe bindings over Google's WebRTC, the
library a browser has. The web side is the browser's
`RTCPeerConnection`, so the two ends make the same offers and use the
same codecs. `webrtc-sys` is the unsafe layer under it and is not used
directly; `livekit` speaks to LiveKit's own server and is not used.

Nothing was fetched to build it:

- The crates are Zed's fork of LiveKit's Rust SDK, which cargo's git
  cache already held (`zed-industries/livekit-rust-sdks`, commit
  `d0e27be0`: `libwebrtc 0.3.26`, `webrtc-sys 0.3.23`). This crate's
  `Cargo.toml` names that commit, so it resolves offline. The root
  `Cargo.toml` lists the same versions from crates.io, which cargo's
  cache does not have.
- Every registry crate they need is in cargo's cache once `hyper-util`
  is held at 0.1.20 in `Cargo.lock` (0.1.21 wants a `base64` that is
  not cached): `cargo update --offline -p hyper-util --precise 0.1.20`.
- The prebuilt WebRTC its build would fetch is
  `webrtc-win-x64-release.zip`, 93,188,956 bytes, from
  `github.com/zed-industries/livekit-rust-sdks/releases/download/webrtc-0001d84-4/`.
  A build of 16 September 2026 had already unpacked it:
  `phikit/target/debug/build/webrtc-sys-d1449d9ba1ad3dc9/out/webrtc-0001d84-4/win-x64-release`
  (`lib/webrtc.lib`, 87 MB, and `include/`). `LK_CUSTOM_WEBRTC` set to
  that folder has the build use it and fetch nothing;
  `target/phiogy_run.ps1 -Rtc` sets it.
- `phikit/.cargo/config.toml` already builds Windows with the C runtime
  built in, which that library needs.

- [ ] the prebuilt WebRTC lies in a build folder: `cargo clean`, or
  clearing `target/debug`, removes it, and the next `-Rtc` build would
  then fetch the file above. Where to keep a lasting copy is the
  user's word
- [ ] `rtc` is off by default, so a plain build joins a call with the
  stand-in. Turn it on by default once the library has a lasting place

Three things the library insists on, each found by its failing and
kept in `src/peer/rtc/link.rs`:

- an offer must say it takes sound and picture. Left unsaid, this
  binding offers to send only, and a later offer turns off what the
  other end was already sending
- a way to reach the other caller must not be added before their
  description is set: the connection then never answers again
- an offer cannot be taken back (a rollback description cannot be
  made), so the end that gives way makes its connection anew. A
  browser never makes this window give way; two of this window do

## The caller

- [x] the system's microphone, through `cpal`: open while in a call
  and not muted, closed while muted. Measured: its level rose from
  0.001 to 0.041 with one short sound in the room
- [ ] the system's camera, through Windows' Media Foundation
  (`rtc/camera.rs`): built, and **never run**. The machine this was
  made on has no camera: the call says "The system gave no camera (it
  has none)" and goes on (`doc/call-no-camera.png`). It wants a run on
  a machine that has one: that its picture reaches a page at the
  camera's size, that its light goes out when turned off
- [x] playing what is heard, on the system's default sound out.
  Measured, not heard by a person: the system's own meter of the sound
  out read 0 before the call and up to 0.09 with the other caller's
  voice played at 0.15 of full
- [x] showing what is seen: each caller's pictures in a tile, through
  appkit's film (`PFilm`); a shared screen in a tile of its own
- [x] sharing a screen: the library's own capturer, the first screen
  (`scene.pick`), at its own size (a page saw 1920 x 1080)
- [ ] more of sharing a screen is a second phase, by the user's word
  (22 November 2026). Several screens at once, or a window in a
  screen's place: a page takes one shared stream from a caller, so
  more needs a change on both sides
- [x] a call begun from a room's title, with the camera or without;
  Mute and End the call among the call's own buttons, as the web's
- [x] a call somebody else began said in the title, with Join
- [x] a client's window: `--at` and where a line is has the window
  reach a line another serves, and listen to the open room's line
  for a call begun there (`src/peer/ring.rs`)
- [x] voice rooms: who is in one, Join and Leave, Mute, and the
  others' sound off and on (`src/part/voice.rs`)
- [x] a voice room one is in shown at the foot of the menu on every
  page the guide is on, as the web's `VoiceBar`: its name, the
  microphone, the others' sound, the way out (`src/part/guide/foot.rs`)
- [ ] who is in a voice room listed live under its row of the rail
- [ ] the bar's Menu row has no rooms in its menu, so no foot either:
  a voice room one is in is not shown while that menu is open
- [ ] in a client's window who has the room open is not listed (that
  is the board of whoever serves the line), and its foot still says
  where a page reaches the line
- [x] a picture of every screen of a call in `doc/scene/`, named by
  who sees it and what they do; `node script/scene.mjs` takes them,
  with the stand-ins, each caller's ramp in a tint of its own
  (`ramp.blue`, `ramp.red` in `data/peer.tsv`)
- [ ] choosing a device that is not the system's default; a device
  plugged in or taken out during a call (the microphone is asked for
  again at each unmute, the others only when turned on)
- [ ] taking this window's own sound out of what its microphone hears
  (echo): the library can, and is not asked to
- [ ] a camera on a system that is not Windows
- [ ] finding a way again when a connection fails (the browser's side
  restarts ICE)
- [ ] it posts each signal on a wire of its own
- [ ] one call at a time: beginning or joining one leaves any other,
  a voice room too


## The line

- [x] who a caller is: behind Phiora the line is told
  (`src/exchange/told.rs`, the heads and the setting in
  `data/line.tsv`): Phiora passes each ask on with who the page belongs
  to and a shared word (`PHI_CALL_KEY`), so a caller has their
  account's name and the line gives no cookie. Reached directly, a
  page is still a guest by the line's own cookie
- [ ] the word is one both sides are started with; nothing changes it
  while they run, and it crosses in the clear between Phiora and the
  line, which must therefore be on the one machine
- [ ] a wire a thread, and a second for a stream: fine for a room, not
  for a server. One loop for every wire
- [ ] a wire is closed after each answer; a browser opens one a post
- [ ] `tiny_http`, which the workspace lists, was not used: it is in
  cargo's cache and not in the lock, and it gives no way to see a held
  stream's page go away, which the line needs to drop it at once
- [ ] a request sent in chunks is not read: a browser and Node send a
  length
- [ ] HTTPS: plain HTTP, which a browser allows on this machine only
- [ ] a guest past the most the line remembers is forgotten, the oldest
  first, even with a line open: its next post is refused
- [ ] no record of joins and leaves (which `philog` is the user's word)

## The window

- [ ] a wake from the line: while the line is open the window draws
  every frame to see who came, since nothing can wake it
- [ ] `Choice` and `fit` are Phigen's, copied: they belong in appkit,
  which this round did not touch
- [ ] the hosts' own marks (`mark` in `host.json`, a cell of a sheet of
  portraits) are not drawn: each place has its icon
- [ ] a room's messages, its unread count, and a person's presence
- [x] a place's headings: its categories, as Phiegg keeps them, over
  their channels, and a channel's own channels under it, as Discord
  shows a server (24 November 2026)
- [x] searching the hosts: Search's Hosts tab, with Phiegg's worlds,
  clubs and shows beside them (24 November 2026)
- [ ] the hand-written places (Phiace, Makers) keep their Text and
  Voice headings, Discord's own defaults
- [ ] Search's worlds, clubs and shows go nowhere when pressed: Phiogy
  has no page of a world yet
- [ ] the running Phiora keeps the seed it was started with: the new
  host tree shows from Phiora after a restart (`npm run seed`)
- [ ] the site is read from a path in this crate's folder
  (`data/app.tsv`, `site`); a program moved elsewhere needs `--site`

## In a browser

- [x] built: `target/phiogy_web.ps1` builds the library for
  `wasm32-unknown-unknown` at low priority (about 10 minutes cold, 20
  seconds after) and makes `web/pkg` with the `wasm-bindgen` that
  wasm-pack keeps on this machine; no warnings (24 November 2026). The
  module is 13 MB
- [ ] opened in a browser and seen: `web/index.html` served (a
  `phiogy-web` entry in `.claude/launch.json`, port 4397), where it
  needs WebGPU; it has no line and reads no site, and says so
- [ ] a site in the browser: read the web Phiogy's files over HTTP
  (`/phiogy/data/`) or from Phiora, as the desk reads them from disk
- [ ] served beside the web Phiogy on phiegg.com

## Not checked

- a camera, on any machine
- what is played, by ear: the system's meter says sound goes out, and
  nobody listened to it
- a real hand's press: Join, Camera on, Mute, Camera off and Leave
  were pressed in the real window by the system's own press messages
  (`target/phiogy_press.ps1`), not by a hand on a mouse

- the window by hand: the pictures open it by flag, and the tests drive
  the same stage. Nobody pressed its rows in the real window
- two machines, two networks, Firefox, Safari, phones, three or more
  callers through this line (the Node line's own notes say the same)
- a long run: the board's size is tested over a thousand pages, the
  line's threads and memory over hours are not

## Tasks: Phiogy as a creators' space (asked 23 November 2026, not built)

The user asked for a tasks module as Microsoft To Do has one (their
picture: My Day, Important, Planned, Assigned to me, Flagged, Tasks,
New list; Grid and List; Sort and Group; Add a task), because Phiogy
is a space creators work in. Its shape is drawn in `ARCHITECTURE.md`
("Tasks, planned") and its folders in `PLAN.md`.

- [ ] `data/task.tsv`: the lists every person has (my day, important,
  planned, assigned, flagged, all), each with its icon, its words and
  the rule that puts a task in it; `doc.task` in `data/keep.tsv`
- [ ] `src/comp/task.rs`: a task (what, done, important, the day it is
  due, how it repeats, its steps, a note, who it is given to, the
  message it came from, the host it belongs to), with no window in it
- [ ] `src/system/task.rs`: add, finish, star, plan, give, and which
  list each task stands in, tested without a window
- [ ] `src/part/task.rs` (`task/`): the page: the lists in the bar's
  menu with how many each holds; the tasks as a grid of cards or as a
  list; a field to add one; sort and group; one task opened beside
  them, in the drawer that people open in
- [ ] a message becomes a task from its controls (a flag), and the task
  leads back to the message
- [ ] a host's own list, which everyone in the host sees: over Phiora,
  as the rooms are
- [ ] FLOW steps and SPECS rows as each lands

## With other people, on other machines (asked 24 November 2026)

What was asked and answered is in [`RUN.md`](RUN.md).

- [x] the line served beyond `localhost`: `--host` and the computer's
  name or address (`exchange::a_line_listens_at_the_name_it_is_given_so_another_machine_reaches_it`)
- [ ] a build that runs on another computer: the site, pictures, icons
  and keeping folder are found through `CARGO_MANIFEST_DIR`; look
  beside the exe first, and a script that gathers one folder to hand on
- [ ] a call between two machines on one network, then two networks
- [ ] a macOS build, on a Mac: Xcode, the Mac WebRTC, a bundle with its
  permissions, signing and notarising (`RUN.md`, 3)
- [ ] Discord: a bot the user adds to their own server, showing the
  pictures and videos of the channels they pick (`RUN.md`, 4); never a
  person's cache, token or pages
- [ ] Phiegg's own people repeat 68 names and 7 handles; renamed there
  (`puikit/app/phiegg/src/data`), the skipped test
  `tree::across_the_app_no_two_people_share_a_name_or_a_handle` passes

## The creators' library (asked 24 November 2026)

Built: the library as two columns, a tree and what the place holds, as
Reality Composer Pro's Project Browser and Blender's file browser
(drives, the person's folders, Phiogy's, folders down to where one is,
Phiegg's collections and clubs, sets); sections that fold and items as
a grid or a list (appkit's form, for every app); the inspector beside an
item (about, parts, colour, finish, attributes, comments); sets made of
a model, each part's colour drawn by weight; Import into the Phiegg
space, a source turned into USD by phiphx; the thing page as Phiegg's;
the hand cursor said by the kit's blocks; eight looks. FLOW 48-54.

Not built, the nearest first:

- [ ] the server receives what a creator brings: an upload route on
  Phiora (`PUT /file/<space>/<name>`, a length, not chunks), kept under
  `/srv/phi-cdn/<user>/`; every source turned into USD there by
  phiphx's converter (or Blender's own exporter where it is installed)
- [ ] the library's collections and clubs read from Phiora's tree
  (`GET /tree?kind=collection,club,pft`, its relations) where Phiora
  answers, and from `shelf.json` where not, as the site is; a set made
  here kept on Phiora as a collection with its items
- [ ] a set's traits beyond colour: a part shown or not, a material, a
  scale, each with its own weights (`data/paint.tsv` is the first)
- [ ] `data/convert.tsv` names `phiphx-convert`: until it is on the
  path a source waits (`cargo run -p phiphx --example convert -- IN OUT`)
- [ ] a folder that changes outside Phiogy is seen when it is chosen
  again (the listing is kept until then); a watch on the folder
- [ ] a large model's tile drawn away from the window; a video's first
  frame as its tile; on a Mac the system asks before Phiogy reads
  Desktop, Documents and Downloads (Info.plist words)
- [ ] the library written as Apple's samples keep theirs
  (`D:/Dowloads/C-DOWNLOADS-NOV-26`: Diorama, PresentingAnArtistsScene):
  an `.rkassets` folder whose root `.usda` references each asset's own
  file, bound to materials under `/Root/Materials`, lit by an image
- [ ] phibot's robot view on the kit's orbit (`pui/core/orbit.rs`)
