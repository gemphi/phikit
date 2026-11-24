# Phiogy - run, test with others, build, Discord

Asked 24 November 2026. Each answer was checked against the code and the
tests on this machine: `target/phiogy_run.ps1 -Verb test` passes 125
tests, 0 fail. Two are skipped on purpose. One is an older test. The
other is the check of Phiegg's own repeated names (see 5).

## 1. Launch on Windows

Build the optimised window (low priority, from `phikit`):

```powershell
powershell -NoProfile -File C:\Users\phiac\Workspace\gemphi\phikit\target\phiogy_run.ps1 -Verb build -Quick
```

Open it with its output kept, as the window the user keeps open is:

```powershell
Start-Process C:\Users\phiac\Workspace\gemphi\phikit\target\main-ui\quick\phiogy.exe -WorkingDirectory C:\Users\phiac\Workspace\gemphi\phikit -RedirectStandardOutput C:\Users\phiac\Workspace\gemphi\phikit\target\phiogy.user.out -RedirectStandardError C:\Users\phiac\Workspace\gemphi\phikit\target\phiogy.user.err
```

A build cannot replace `phiogy.exe` while that window is open: close it
first. The window serves the line on `localhost:4310` itself.
Useful flags: `--room lobby`, `--look dark`, `--port 4300`, `--at
HOST:PORT` (be a client of a line someone else serves), `--host NAME`
(serve the line at that name, see 2), `--line` (the line alone, no
window), `--forget` (keep nothing). Real sound and pictures need the
`rtc` build: `target/phiogy_run.ps1 -Verb build -Rtc` (it uses the
prebuilt WebRTC already on this disk and fetches nothing).

## 2. Testing with other people

**Works now, on one machine.** Several callers meet on one line: the
window, its `--caller` windows, and web pages pointed at the line. They
see who has a room open and who is in its call. Calls run over WebRTC
(sound, camera, a shared screen, Mute, End). The tests prove this with
pages on the same machine (`exchange`, `peer`, `pill`, `ring`, `voice`).
A whole call between this window and a Chrome page was measured both
ways. The web Phiogy at `https://phiegg.com/phiogy/` already carried
calls between two browsers through the server's line.

**Not tried:** two machines, two networks. Nothing has run on a second
computer.

**Missing for people on other Windows machines**, in order:

1. **A line others can reach.** It listened on `localhost` only. Now
   `--host DESKTOP-KGBAEQV` (this computer's name), or its address
   `192.168.1.101`, serves it on the home network. Tested:
   `exchange::a_line_listens_at_the_name_it_is_given_so_another_machine_reaches_it`.
   Windows Firewall must let in TCP 4310 (the user's decision: an
   inbound rule for `phiogy.exe`, private networks only).
2. **A build that runs on another computer.** The exe finds its site,
   pictures, icons and keeping folder through the source folder named
   at build time (`CARGO_MANIFEST_DIR` in `app.rs`, `hosted.rs`,
   `run.rs`, `exchange/person.rs`). On a friend's PC those paths do not
   exist. Needed: look beside the exe first, and a script that copies
   the exe, `data`, `asset`, the site (`puikit/app/phiogy/public`, 6 MB)
   and Phiegg's portraits and worlds (about 10 MB) into one folder.
3. **Real sound and picture by default.** `rtc` is off by default
   (`TODO.md`, "The native peer"). The prebuilt WebRTC sits in a build
   folder that `cargo clean` would remove. Keep a copy somewhere
   lasting, then turn `rtc` on.
4. **Same network, then the internet.** Without a relay, WebRTC
   reaches the same network only. Over the internet: the relay on the
   server (coturn) needs UDP 3478, TCP 3478 and UDP 49152-49407 opened.
   The native window also needs HTTPS to reach
   `https://phiegg.com/api/call/` (it speaks plain HTTP today).
5. **Sign-in.** On the bare line everybody is a guest. Behind Phiora a
   caller has their account's name. The sign-in Phiora accepts has not
   been tried against the real Phiora.

So the first test with a friend: both on the same Wi-Fi, both with
this workspace (until step 2 is done), one runs `--host` with that
computer's name, the other `--at NAME:4310`, both built with `-Rtc`.
The easiest test today: the web Phiogy on phiegg.com in two browsers.

## 3. Builds

**Windows.** `-Verb build -Quick` gives the optimised build in
`target/main-ui/quick/phiogy.exe`. On 24 November it compiled into
`target/main-ui/quick/deps/phiogy.exe`. It did not replace the open
window's `phiogy.exe`, which Windows locks. Close the window and run the
build again: it then only copies the file. `-Rtc` adds WebRTC. The
first `--host` run makes Windows Firewall ask about `phiogy.exe`. Before handing it to anyone else, it needs step 2 of section 2.

**macOS.** Needs a Mac; it cannot be built from here. It needs:

- Xcode (the full app, for the Metal shader compiler) and Rust for
  `aarch64-apple-darwin` (and `x86_64-apple-darwin` for Intel Macs).
  gpui draws with Metal (`gpu/gpui_macos`); `MACOSX_DEPLOYMENT_TARGET`
  is already 10.15.7 in `.cargo/config.toml`.
- With `rtc`: the prebuilt WebRTC for macOS from the same release as
  Windows' (`zed-industries/livekit-rust-sdks`, tag `webrtc-0001d84-4`,
  `webrtc-mac-arm64-release.zip`), with `LK_CUSTOM_WEBRTC` set to it.
  Downloading it is the user's decision. `cpal` gives the microphone
  and speakers through CoreAudio. The camera is Windows-only
  (`rtc/camera.rs`): on a Mac the call says it has no camera.
- An app bundle (`Phiogy.app`) with an `Info.plist` that says why it
  wants the microphone and camera (`NSMicrophoneUsageDescription`,
  `NSCameraUsageDescription`). Screen sharing needs the Screen Recording
  permission.
- To run on other Macs: an Apple Developer ID, `codesign --options
  runtime` with the audio-input and camera entitlements, then
  `notarytool submit` and `stapler staple`.
- The same paths problem as Windows (section 2, step 2).
- The caption: the window's own top row is drawn on Windows only
  (`screen/mod.rs`, `bare`); on a Mac the system's buttons stay.

## 4. Discord's pictures and videos

**Where they are on a Windows PC.** Discord keeps nothing in tidy
folders. Pictures and videos posted in Discord live on Discord's servers
(`cdn.discordapp.com`, `media.discordapp.net`). Their links now expire
after about a day and are renewed by Discord's own app or API. The
desktop app keeps a browser cache of what it showed under
`%AppData%\discord\Cache\Cache_Data`: numbered pieces with no names, not
meant to be read. A file somebody chose to save is in their own
Downloads folder. A 3D session, a game or screen shared with Go Live,
is a live stream. It is not kept anywhere, and no API hands it out.

**What Phiogy must not do.** Read another person's Discord cache or
files, use somebody's Discord login token, run a "self-bot", or scrape
Discord pages. All of these break Discord's terms and the friends'
privacy. Phiogy has none of it and will not get it.

**What Discord allows.**

- **A bot the user adds to their own server.** The user makes an
  application at `discord.com/developers`, adds a bot, turns on the
  Message Content intent, and invites it to their server with *View
  Channel* and *Read Message History* on the channels they choose.
  Through the API (`GET /channels/{id}/messages`) it then reads those
  channels' messages and their `attachments` (pictures and videos, with
  type and size), and hears new ones live. That is what Phiogy would
  show: in a room of a host called after the Discord server, the
  viewer's pictures and a video player.
- **Ask first.** Tell the friends in the server that the bot mirrors
  shared pictures into Phiogy. Only bring channels the user picks. Drop
  a picture when Discord says it was deleted. Keep files only while
  they are shown, and fetch a fresh link from the API when an old one
  has expired.
- **The user's own data.** Discord's *Request all of my data* (Settings,
  Data & Privacy) gives a person their own messages and the links to
  their own attachments. Phiogy could import a package the user gives
  it. That is the user's own data only, not friends'.
- **Live 3D sessions.** A bot cannot receive Go Live video. The allowed
  way is for friends to share their screen in a Phiogy call, which
  Phiogy already carries (Share a screen).

Nothing of this is built. The bot's token would be kept in a system
setting (as `PHIOGY_SECRET` is), never in a file or on a command line.
Making the application and inviting the bot are the user's steps.

## 5. The people and the tree

There is no fixed set of 64 profiles in Phiogy. The number is
the fewest people a host (a place) seats: `space.least` 64 in
`puikit/app/phiogy/server.json`, up to 256. Every place shows 65 to
256 people. The check below covers all 1,737 people and all 276
places. It is `src/test/tree.rs`.

| Test | Before | Now |
|---|---|---|
| every person has a name, a handle, a face, a role and a bio (all but "you", who writes their own) | passed | passes |
| inside every place, no two people share a face, a name or a handle | 1,692 repeats in 262 places | passes |
| every place has rooms, a room to call in (voice) and at least 64 people; every page shows a room that exists; a call holds callers | 103 places had no room to call in | passes |
| across the whole app no two people share a name or a handle | - | skipped: fails on Phiegg's own data |

**What changed** (`puikit/app/phiogy/server.py` and `server.json`, then
rerun):

- A place now seats people who wrote there first. Then it passes over
  anyone whose face, name or handle is already seated. Nobody's face or
  name changed, so a face in Phiogy is still that person's face in
  Phiegg.
- Every host Phiegg gave no voice room, and every village space, gets a
  **Lounge** to call in. Its name and topic are `call` in `server.json`.
  1,141 rooms now; 1,042 before.

**What is left, in Phiegg's data, not Phiogy's.** Phiegg's
own people repeat 68 names and 7 handles across different persons, for
example two "Hugo Rainier", founders of Moss Row and of Wren House,
both `hugo.rainier`. In 44 cases two such people both wrote in the same
place, so a place shows both. Fixing it means renaming people in
`puikit/app/phiegg/src/data` (`core.json`, `agents.json`). Other
sessions are editing that folder, so it was left alone. There are also
1,737 people and 768 portraits (12 sheets of 64), so a face can be
unique within a place but not across the whole app without more
finished portraits.

**Levels of the tree**, as Phiogy has them:

| Level | Where | Count |
|---|---|---|
| hosts (areas of the window) | `data/host.tsv` | 6 |
| places (servers) | `host.json` catalogue + the rail's own | 270 + 7 |
| rooms (channels) | `room.json`, `host.json` | 1,141, each place 1 or more |
| rooms to call in (voice) | every place but Direct | 1 or more each |
| calls | on the line, live | 8 callers a call, 256 pages a line (`data/limit.tsv`) |
| screens | pages of `phiogy.json` + `data/page.tsv`; shared screens in a call | 15 + 8 |
| people | `room.json` | 1,737, 65-256 a place |
| messages | `room.json` | 3,669; the 32 collection spaces are empty until people join |

## What works now and what is planned

Works (tests pass; many have pictures in `doc/`): the window and its bar,
rooms and their messages, faces and pictures, the viewer, writing,
answering, liking and pinning, search, a person beside a room, hosts
(join, leave, make, remove), friends and direct conversations, shops
with a bag and play money, the account (Phiora's demo members, a
handle and password, sessions, a copy of one's data), four looks, the
line, calls with Mute, camera, screen sharing and End, voice rooms,
being a client of another's line (`--at`), and now serving the line
beyond localhost (`--host`).

Planned (`TODO.md`): messages that reach the other people of a room;
a relay for calls across networks; a build that runs on another
computer; `rtc` on by default; HTTPS; the camera seen working (this
machine has none) and a camera on a Mac; echo cancellation; picking a
device; a tasks module; the Discord bot of section 4.
