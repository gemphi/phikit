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
| Room stream | Message + Send | OWN DOC: nobody else receives it |
| | Post a picture | OWN DOC: keeps a `file:///` path, never uploads |
| | Reply, Like and marks, Pin | memory / OWN DOC |
| Person beside the room | Back, Message, Follow, Mute | OWN DOC (follow and mute reach nobody) |
| | Profile tabs with layout, filter, order | NAV / memory |
| Call | Mute, Camera, Share a screen, End | default build: stand-in engine, sends nothing (STUB); real media needs the `rtc` build |
| Voice room, guide foot | Join, Leave, Mute, Sound off | as Calls |
| Guide (Rooms) | Recent / Favorites / Older | LOCAL ONLY |
| | A room's row | NAV, marks seen/read (OWN DOC) |
| Hosts | Name your host; Create my own, Collection, Studio, Friends, Club | OWN DOC: the host exists only for its maker |
| | Remove (made host); Join / Leave (catalogue host) | OWN DOC |
| | Search hosts | LOCAL ONLY |
| Friends, People | Add friend, Accept, Decline, Cancel, Remove | OWN DOC: the other person never gets it |
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
| Creating hosts | the maker's own `host-made` document | the app never calls Phiora's `POST /tree/item`; nobody else can see or join |
| Quests | not built | no control, no code |
| Pools | not built | no control, no code (the web makes them through `POST /tree/item`) |
| Any Phiegg record (create) | not built | no create sheet |
| Editing a club | not built | the record page is read-only; no `PATCH /tree/item` |
| Editing the profile | not built | the profile's head is the data file's Guest |
| Uploading files | local copy only | Phiora has no upload route; posted pictures are local paths |
| Messages, friends, follow reaching others | the sender's own document | there is no shared room or request on the server yet |
| Calls between people | this window's own line | the line is Phiogy's, not the server's; the default build carries no sound or picture |
| Anything kept, without Phiora | this machine only | kept documents go to Phiora only when it answered at launch |

## Why pages are not laid out right

Only the Library (tree, library, inspector) and a person's tabs are the
kit's form, laid out from `data/form.tsv` with its grids, layouts, filters
and folds. The rest:

- **Phiogy's own list** (`src/part/list.rs`, items written in Rust): Hosts,
  Account, Friends and People, Shops and the bag, Search, Record. These
  have no layout switch, no folds, no drawers.
- **Drawn by hand**: the title, status, room, stream, people beside the
  room, the person's card, calls, voice, the guide, the shop's shelf and
  thing page, the viewer, the model space.

So they cannot pick up what the kit gives every form, and each looks the
way its own code draws it.

## Order of work

1. Pages onto the kit's form, one at a time: Hosts (create a host as a
   drawer with help), Shops' thing page (tabs and folding sections, as the
   web's), Friends and People, Record, Account, Search.
2. Creating and editing through Phiora (`POST` and `PATCH /tree/item`):
   hosts that others can join, quests, pools, clubs, the profile.
3. Upload (`POST /file`, SHARED-LIBRARY.md), then posted pictures and the
   library's Team place.
4. Rooms, friends and calls through the server, the `rtc` build.
