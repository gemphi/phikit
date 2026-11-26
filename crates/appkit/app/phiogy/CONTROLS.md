# Phiogy: every page's controls, and what really works

Audit of the desktop Phiogy, 25 November 2026, read from the code (the
handlers named after each). The status words:

- **WORKS**: does the whole job, the server step included.
- **NAV**: works; only moves between views or opens one, nothing kept.
- **OWN DOC**: kept in the person's own document (`src/kept.rs`): the
  local `state.json`, and `PUT /state/<name>` to Phiora **only when
  Phiora answered at launch** (`src/run.rs`). Nobody else ever sees it.
- **LOCAL ONLY**: this machine's files or memory only.
- **STUB**: shows but does nothing real. **MISSING**: the web has it, this
  app does not.

## Pages and their controls

| Page | Controls | Status |
|---|---|---|
| Bar and menu | Phiogy mark, Rooms, Hosts, Shops, Library, Menu, a host's row, Start, Account, Profile | NAV |
| | The eight looks | LOCAL ONLY |
| Profile (appkit) | Mode, Surface, Grounds | LOCAL ONLY |
| | Name and mail at its head: always "Guest / not signed in" | STUB |
| Title over a room | Search, Search this room, Pinned | NAV / memory |
| | Add to favorites | OWN DOC |
| | Call, Call with camera, Join | see Calls |
| Room stream | Message + Send | WORKS through Phiora for a signed-in person (a `message` record, read by the room every 4 s); OWN DOC for a guest |
| | Post a picture | OWN DOC: keeps a `file:///` path, never uploads |
| | Reply, Pin | memory / OWN DOC |
| | Like and marks | WORKS through Phiora (a `mark` record per person and message); OWN DOC for a guest |
| Person beside the room | Back, Message, Follow, Mute | Follow is told to Phiora (a `follow` record) and nothing shows it yet; Mute is OWN DOC |
| | Profile tabs with layout, filter, order | NAV / memory |
| Call | Mute, Camera, Share a screen, End | default build: stand-in engine, sends nothing (STUB); real media needs the `rtc` build |
| Voice room, guide foot | Join, Leave, Mute, Sound off | as Calls |
| Guide (Rooms) | Recent / Favorites / Older | LOCAL ONLY |
| | A room's row | NAV, marks seen/read (OWN DOC) |
| Hosts | Name your host; Create my own, Collection, Studio, Friends, Club | WORKS through Phiora (a `host` record others can Join); OWN DOC for a guest |
| | Publish a club, Publish a PFT | WORKS through Phiora (the kit's form; records Phiegg's web reads) |
| | Remove (made host); Join / Leave (catalogue host) | OWN DOC |
| | Search hosts | LOCAL ONLY |
| Friends, People | Add friend, Accept, Decline, Cancel, Remove | WORKS through Phiora with the seeded people who are Phiora's persons (a `bond` record the other answers); OWN DOC with the rest and for a guest |
| | Message, a person's line | NAV + OWN DOC |
| Search | Field, tabs (All, Rooms, People, Hosts, Worlds, Clubs, Shows), Popular, See all | reads Phiora's `/tree` or Phiegg's files |
| | A card | NAV |
| Record (world, club, show) | Go to its host, related cards, its rooms | NAV |
| | Edit | MISSING |
| Shops | Search, Open / Close a shop, Add to bag, Like, Put on (theme) | OWN DOC (wear never reaches Phiora's theme route) |
| | Bag: Pay from wallet | sent to Phiora as play money; the answer is not read until the next launch |
| Account | Sign in, demo sign in, sign out, sessions, sign out others, export | WORKS when Phiora is up |
| | Create account, email, password reset or change, delete, revoke one session | MISSING |
| | Wallet | display only |
| Viewer | Zoom, previous, next, save a copy, open in system | NAV / LOCAL ONLY |
| | Share: copies a local path | LOCAL ONLY |
| | Sketch, Text, Undo, Redo, Save marks | LOCAL ONLY |
| Library | Tree, Back, Forward, Up, crumbs, layout, Show, Order | NAV / memory |
| | Import: copies into a local folder, converts Blender files locally | LOCAL ONLY |
| | Make a set, colours, finish, comment | LOCAL ONLY |
| Model | Reset, four views, wireframe, drag, wheel | LOCAL ONLY |

## What does not work, and why

| Asked for | Where it stops | Why |
|---|---|---|
| Creating hosts | done (26 November): `host` records | joining copies the rooms; what is said in them is the room's `message` records |
| Quests | not built | no control, no code |
| Pools | not built | no control, no code (the web makes them through `POST /tree/item`) |
| Any Phiegg record (create) | not built | no create sheet |
| Editing a club | not built | the record page is read-only; no `PATCH /tree/item` |
| Editing the profile | not built | the profile's head is the data file's Guest |
| Uploading files | local copy only | Phiora has no upload route; posted pictures are local paths |
| Follow shown to the followed, pins, mutes, uploads | the sender's own document or a record nothing reads | the follower count is the site's number; no upload route |
| Calls between people | this window's own line | the line is Phiogy's, not the server's; the default build carries no sound or picture |
| Anything kept, without Phiora | this machine only | kept documents go to Phiora only when it answered at launch |

## Why pages are not laid out right

The Library (tree, library, inspector), a person's tabs, the Hosts (and
their drawers), a shop's thing page and the column beside a room (the
host, its line and its tabs) are the kit's form, laid out from
`data/form.tsv` with its grids, layouts, filters and folds. The rest:

- **Phiogy's own list** (`src/part/list.rs`, items written in Rust):
  Account, Friends and People, Shops and the bag, Search, Record. These
  have no layout switch, no folds, no drawers.
- **Drawn by hand**: the title, status, room, stream, the person's card, calls, voice, the guide, the shop's shelf, the
  viewer, the model space.

So they cannot pick up what the kit gives every form, and each looks the
way its own code draws it.

## Order of work

1. Pages onto the kit's form, one at a time: Hosts (done, a host made
   and opened in drawers), Shops' thing page (done: picture held at the
   left, tabs of folding sections), Friends and People, Record, Account,
   Search.
2. Creating and editing through Phiora (`POST` and `PATCH /tree/item`):
   hosts that others can join, quests, pools, clubs, the profile.
3. Upload (`POST /file`, SHARED-LIBRARY.md), then posted pictures and the
   library's Team place.
4. Rooms, friends and calls through the server, the `rtc` build.
