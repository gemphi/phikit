# Phiogy

The app that carries the calls, on appkit, made the way Phigen is. This
has the rules of a call, the line pages find each other on, a window
that shows who is on it, and, since its second round, the window as a
caller itself: it joins a room's call over the line as a page does and,
built with WebRTC, sends and takes sound and picture: the system's
microphone, its camera where it has one, a screen it shares, and what
the others send is played and shown in tiles over the room.

```bash
cargo run -p phiogy
```

The line alone, with no window (it says where it listens):

```bash
cargo run -p phiogy -- --line --port 4300
```

A caller with no window, which says each second what it hears and sees
(with the real WebRTC built in, see "Building with WebRTC"):

```bash
cargo run -p phiogy --features rtc -- --caller phiogy/lora --port 4311
```

Pictures of the window: [`doc/room.png`](doc/room.png), and in a call
with another caller, [`doc/call-rtc.png`](doc/call-rtc.png).

## What it is made of

| Folder | Holds |
|---|---|
| `data/` | everything that is not code: the limits of a call, what a name may be, the words on the wire, what the line refuses and says, where it listens, the heads of its answers; and the window: its name, spaces, hosts, regions and parts, the rows of its activity bar and their menus, its user, its look, its words |
| `asset/` | files only Phiogy has: its mark. Its other icons and its face are the ones every app shares |
| `script/logo.py` (`shape.py`, `sheet.py`) | draws the mark on the measures of Phiogy's face, from `data/logo.tsv` and the corner options of `data/logo-corner.tsv`, into `asset/icon/`, `doc/logo/`, `doc/logo.html` and `doc/logo-option.png` |
| `script/line.mjs` | runs the Node line's own tests, unchanged, against this line |
| `script/phiora.mjs` | starts the real Node Phiora and has the program say what it read: of Phiora, after an edit made in Phiora, of nobody, and of a Phiora that is closed; and write a message, which is then looked for on Phiora |
| `script/peer.mjs` | the web's own two-Chrome call check with one Chrome replaced by this program, as a caller with no window or (`WINDOW=1`) as the window itself; each side measured |
| `src/entity/` | what there is in a call: `room`, `caller`, `line` |
| `src/comp/` | what they carry and what the rules read and say: `limit`, `roster`, `signal`, `ice`, `wire`, `fault`, `note` |
| `src/system/` | the rules: `join`, `pass`, `leave`, `beat` |
| `src/board.rs` | every room with a line open: a world of `phiecs` the rules change and the window reads |
| `src/exchange/` | the line, served: `set` (its settings), `ask` and `answer` (HTTP), `guest` and `told` (who a page is), `hub` (what the wires share), `route` (the routes), `person` (an app's document, `me`, a person's documents) |
| `src/peer/` | this window as a caller: `line` (a room's line as a page holds it), `mesh` (a connection to each other caller, and who gives way), `engine` and `stand` (what a connection is asked, and one that carries nothing), `call` (a call on a thread of its own), `rtc/` (the real WebRTC, with the `rtc` feature) |
| `src/site.rs`, `src/site/read.rs` | the web Phiogy's own hosts, rooms, people's faces and pages, and how they are read from its files |
| `src/phiora.rs` | Phiora asked as a page asks it, over plain HTTP: an app's document, who the one asking is, and their own documents, read and kept |
| `src/kept.rs`, `src/hour.rs` | a person's own documents, in their folder and on Phiora; the hour on their clock, for what they write |
| `src/hosted.rs` | where a picture's key is on disk: the public folder of the app it names |
| `src/app.rs` | `Phiogy`, which is `PApp`: the four sets read from `data/`, and the view of each part |
| `src/part/` | the parts: the room's `title` with the tools that begin a call and the pill that joins one (`join`); a call over the stream (`call`) and a voice room (`voice`); the `room`, which is what was said (`stream`) and who is `present`; the `viewer` of a posted picture; the line's `status`; the `venue` list; the `guide` in the bar's menu |
| `src/choice.rs`, `fact.rs` | what a person chose, kept; what is true now, as the profile shows it |
| `src/run.rs`, `main.rs`, `web.rs` | opening it: the one way, and its two starts |
| `src/test/` | the tests, in the same folders |
| `web/` | the page a browser loads it into; `pkg/` is built, not kept |

The diagrams are in [`ARCHITECTURE.md`](ARCHITECTURE.md), the folders
and the order of work in [`PLAN.md`](PLAN.md), each piece and its test
in [`SPECS.md`](SPECS.md), what a person does in [`FLOW.md`](FLOW.md),
and what is left in [`TODO.md`](TODO.md). How to launch it, test it with
other people, build it for Windows and macOS, and what Discord allows:
[`RUN.md`](RUN.md).

## What it does

- **The rules of a call**, with no network in them. A page opens a
  room's line and is told who is in the room's call. It joins, and
  every page of the room is told. Two who are both in the call pass
  each other signals through the line, in the order sent. It leaves, or
  its page closes, and those left are told. A page belongs to whoever
  opened its line. A call holds 8 and a room's line 256 pages, and a
  page that took nothing for a minute is dropped by the beat: each
  number is a row of `data/limit.tsv`.
- **The line.** `GET /call/APP/ROOM?peer=ID` is a room's line, an event
  stream; `POST /call/APP/ROOM` joins, leaves and sends. They are the
  routes of the Node line (`puikit/app/phiora-server/src/call.mjs`),
  with its words, its statuses, its cookie and its answers to a
  browser on another port, so the web pages speak to this line
  unchanged. It listens on `localhost:4310` (`data/line.tsv`), at each
  address that name stands for.
- **The window.** One bar at the left: Phiogy's mark, Rooms and Hosts
  at its top, Menu at its middle, the user at its foot. Beside it the
  places a person joined with their rooms. They are asked of Phiora
  first, as a page asks it (`GET /app/phiogy`, where `data/phiora.tsv`
  says), and read from the web Phiogy's own file (`data/app.tsv`,
  `site`) where Phiora does not answer; the profile says which it was.
  The room that is open says who is
  on its line now: who is in its call, who only has it open, and under
  them its own people. The Hosts space lists every place the site has.
  The window serves the line itself, so a page that opens a room is
  seen in it at once.
- **What Phiora answers, beside a call.** `GET /app/KEY` is an app's own
  document, read from the app's file each time; `GET /me` says who the
  page belongs to; `GET` and `PUT /state/NAME` keep a person's own
  documents while the line runs. So the web Phiogy pointed at the line
  gets its config there. The wallet, the shop and accounts are not here.
- **A room as the web has it.** The bar's menu is the web's own `nav`:
  each place, its Text and Voice headings, its rooms. A page is its
  blocks, as the app's file writes them: a block that is a room shows
  what was said in it (who, when, the words, what it answers, its marks,
  its pictures by name, a pin), the newest at the foot, with the room's
  people beside it at a desk and over it in a hand. A page added to the
  file opens with no code changed; a block of a kind not drawn yet is
  named.
- **Pictures, the web's own.** A person's face beside what they said,
  in a direct room's title and in the rail; a place's mark in the
  hosts; a picture posted in a room, drawn under its words. They are
  the web's finished files, found by the keys the web names them by
  (`phiegg:worlds/...`; `data/asset.tsv` says where each app's public
  folder is), drawn with appkit's picture mark.
- **The viewer, the web's.** A press on a posted picture opens it over
  the whole window, with no head of a panel's: who posted it and when
  at the top left; its tools and the way out at the top right; the
  picture whole; the one before and after by an arrow or an arrow key;
  a strip of every picture of the room. The tools (`data/tool.tsv`):
  larger and whole again, which a press on the picture does too, the
  larger one moved with the wheel; share, which copies where the
  picture's file is; save, which asks in the system's own dialog where
  a copy is to be kept; open, which opens the file in the system's own
  program for pictures. What a control is called is shown by it while
  the pointer is over it.
- **A room's tools, the web's.** At the stream's foot a field: what is
  written there is said in the room at once (Enter, or the control
  beside it). The message under the pointer, or the one pressed last,
  shows its controls: answer it, like it, pin it. A press on a mark
  gives it or takes it back. In the title: search the room, and list
  what is pinned in it; a press on a pinned line or on what a message
  answers brings that message into sight. A press on a face puts that
  person beside the room: who they are, what to do (write to them,
  follow them), the spaces they are in, who they are linked to, and
  their pictures, which open in the viewer.
- **Pages around the rooms, the web's.** What the window is opened on
  is a page: a room, a room of a host the person made, a direct
  conversation with anybody, or one of the pages that are no room
  (`data/page.tsv`). The rail lists what there is to go to.
  - *Hosts*: make one from a template (its rooms under their
    headings), remove it; search every host there is, join and leave.
  - *Friends* and *Find or start a conversation*, at the head of the
    direct list: who is a friend, who asked, whom the person asked,
    and a search of everyone; a conversation that was begun stays in
    the rail.
  - *Shops*: pick a shop, which puts it in the rail with its own menu;
    its things in a grid; a thing's own page; the bag, paid from the
    wallet. It is play money: nothing real is paid.
  - *Account*, a drawer at the right with one heading: who the person
    is, their wallet, the look, and being somebody to Phiora: one of
    its demo members, or signed in with a handle and a password, which
    is drawn as marks; signed in, the sessions signed in as them, the
    others signed out, and a copy of everything Phiora keeps about
    them.
- **Also in a room:** a picture of this disk posted as a message (the
  system asks which file), and a person muted, whose messages are not
  shown until they are unmuted. **In a shop:** a theme shows its own
  colours, and one that is bought is put on: the window's light takes
  its colour, as `data/wear.tsv` says.
- **What a person keeps.** What they wrote, the marks they gave, what
  they pinned, the rooms they opened, whom they follow, the hosts they
  joined and made, their friends, the shops they opened, their bag and
  what they bought are their own documents, spelled and named as the web's pages keep them: in
  their folder for Phiogy and, with Phiora answering, on Phiora under
  a session Phiora gives the window, so a page signed in as the same
  guest sees the same (`src/kept.rs`, `data/keep.tsv`).
- **A caller.** Call and Call with the camera, two tools at the right
  of a room's title, put the window in the room's call: it opens the room's line over HTTP as a
  page does, joins, and keeps a connection to each other caller. When
  both ends offer at once the one with the lesser name gives way, as
  the browser's side does (`puikit/app/phi-kit/src/peer.ts`), so a page
  and this window meet. While somebody else is calling in the room the
  title says who and has Join where the two tools were; a window
  started with `--at` is a client on a line another serves, and hears
  of such a call by listening to the room's line (`src/peer/ring.rs`).
  The room
  lists this window among the callers and says of each other caller how
  far the connection has come, whether they are muted, how many
  pictures they sent and how loud they are.
- **A call seen and heard.** In a call the room shows it over the
  stream, as the web's call stands there: this window's own tile, the
  screen it shares, each other caller's camera or face, a screen they
  share, and the room's person ringing; under them its buttons, as the
  web's call has them: Mute, Camera on, Share a screen, End the call
  (`src/part/call.rs`). A voice room has, in the stream's place, who
  is in it, Join and Leave, Mute, and Sound off for the others
  (`src/part/voice.rs`); while a person is in one it stands at the
  foot of the menu on every page, with the microphone, the others'
  sound and the way out (`src/part/guide/foot.rs`). `node script/scene.mjs` takes a picture of
  every screen of a call into `doc/scene/`, each named by who sees it
  and what they do (`host-receive-all.png`). The pictures move through appkit's
  film. Sound and pictures are the system's own (`data/peer.tsv`:
  `device system`): its default microphone, closed while muted, its
  first camera, one of its screens, and its sound out. What the system
  does not give is said, with why, and the call goes on. `--set
  device=stand` sends a tone and a grey ramp and plays nothing: the
  tests and every picture in `doc/` are made so, and nothing of a room
  or a screen is kept.
- **The asset library**, a row of the bar (`src/library*`,
  `src/part/library*`, `src/part/figure.rs`), as Reality Composer Pro's
  Project Browser and Blender's file browser are: two columns, a tree
  (Volumes, System, Phiogy's own, the Folders down to where one is,
  Phiegg's Collections and Clubs, Sets) and what the place holds (where
  it is, Up and Import, what to show and in what order, the items as a
  grid or a list, what the place is), every section folding. An item
  opens beside the inspector: a model in 3D, its parts' colours, a
  finish, its attributes, comments. A set made of a model draws each
  part's colour by weight, as a 10,000-piece collection is made from
  its traits. Phiogy shows USD only; a Blender file brought in is turned
  into USD by phiphx. Phiegg's collections and clubs come from puikit's
  `app/phiogy/library.py`. All of it the kit's form, fed by
  `data/form.tsv` and `data/listing.tsv`.
- Nothing has a fill of its own: every surface wears appkit's light,
  and every colour and size is `data/theme.tsv`. Four looks, kept with
  the room that was open.

## What is and is not in code

Nothing a person reads, no limit, no name on the wire, no status and no
address is in the code: each is a row of a table in `data/`, and the
tests change a table to prove it. What the code keeps are the names of
those rows, as it keeps the names of the theme's tokens.

## Building with WebRTC

The real library is behind the feature `rtc`, which is not on by
default. Nothing is fetched to build it on this machine: the crates are
in cargo's caches, and the prebuilt WebRTC an earlier build unpacked is
still on disk. `target/phiogy_run.ps1 -Verb build -Rtc` points the
build at it (`LK_CUSTOM_WEBRTC`) and builds offline. Without the
feature the window still joins a call, carried by a stand-in that
sends and hears nothing, and says so. [`TODO.md`](TODO.md) has what
the library is, where its file would come from, and its traps. The
feature also builds `cpal` (the microphone and the sound out) and
`windows` 0.54 (the camera), both from cargo's cache.

## Not in this round

A camera seen working (the machine this was made on has none), a
relay and a forwarding unit, messages that reach the other
person, and the route taken over on the server in the cloud:
[`TODO.md`](TODO.md) has each.
