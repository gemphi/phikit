# Phiogy - architecture

Diagrams only. The words are in [`PLAN.md`](PLAN.md) and [`SPECS.md`](SPECS.md).
Solid boxes exist and are tested. Dashed boxes are not built: they are
what [`TODO.md`](TODO.md) lists.

## The layers

```mermaid
graph TB
    subgraph data["data/: everything that is not code"]
        d_rule["limit · name · fault · wire · ice"]
        d_line["line · head · status"]
        d_win["app · space · layout · host · region · part · modal ·<br/>activity · menu · user · theme · look · text · icon · site"]
    end
    subgraph rule["the rules of a call: no network"]
        r_entity["entity/<br/>room · caller · line"]
        r_comp["comp/<br/>limit · roster · signal · ice · wire · fault · note"]
        r_system["system/<br/>join · pass · leave · beat"]
        r_board["board.rs<br/>a world of phiecs"]
    end
    subgraph line["the line: exchange/"]
        x_route["route<br/>GET stream · POST · OPTIONS"]
        x_hub["hub<br/>one lock, notes handed out in order"]
        x_http["ask · answer · guest · set"]
    end
    subgraph win["the window"]
        w_app["Phiogy: PApp<br/>app.rs"]
        w_part["part/<br/>title · join · present ·<br/>status · venue · guide · search"]
        w_peer["peer/<br/>the window as a caller"]
        w_site["site.rs<br/>the web Phiogy's hosts and rooms"]
    end
    kit["appkit<br/>the window, the bar, the profile, the light"]
    ecs["phiecs<br/>words, relations, aspects"]
    page["the web pages<br/>puikit: call.ts, peer.ts"]
    d_rule --> r_comp
    d_line --> x_http
    d_win --> w_app
    r_entity --> r_board
    r_comp --> r_system --> r_board
    r_board --> ecs
    x_route --> x_hub --> r_system
    x_http --> x_route
    page --> x_route
    w_part --> r_board
    w_site --> w_part
    w_app --> w_part
    w_part --> w_peer
    w_peer --> x_route
    w_app --> kit
```

## The rules: entity, comp, system

```mermaid
graph LR
    subgraph entity["entity: what there is"]
        e_room["Room<br/>key: app/room"]
        e_caller["Caller<br/>user · name · handle"]
        e_line["Line<br/>peer · joined · seen · turn"]
    end
    subgraph comp["comp: what is read and said"]
        c_limit["Limit<br/>caller · watch · beat · gone · send"]
        c_roster["Roster<br/>who is in a call"]
        c_signal["Signal, Post<br/>what a page sends and asks"]
        c_note["Note: Hold + Tell<br/>First · Roster · Signal · Beat · End"]
        c_fault["Fault<br/>why the line refuses"]
    end
    subgraph system["system: what changes the board"]
        s_open["join::open"]
        s_join["join::join"]
        s_pass["pass::pass"]
        s_leave["leave::leave · close"]
        s_beat["beat::beat · seen"]
        s_post["post"]
    end
    s_post --> s_join
    s_post --> s_leave
    s_post --> s_pass
    s_beat --> s_leave
    s_open --> e_line
    s_join --> e_line
    s_leave --> e_line
    e_line -- "on" --> e_room
    e_line -- "of" --> e_caller
    c_limit --> s_open
    c_limit --> s_join
    c_limit --> s_beat
    c_signal --> s_pass
    s_open --> c_note
    s_join --> c_note
    s_pass --> c_note
    s_leave --> c_note
    s_beat --> c_note
    s_open --> c_fault
    s_join --> c_fault
    c_roster --> c_note
```

## The board as a world

```mermaid
graph TB
    b1["a page opens phiogy/lobby as peer-a"]
    b2["word slot.1<br/>aspect Room: phiogy/lobby"]
    b3["word slot.2<br/>aspect Line: peer-a, out of the call"]
    b4["word slot.3<br/>aspect Caller: guest_1"]
    b5["the page closes"]
    b6["the three words are forgotten<br/>and kept as spare"]
    b7["the next page, room and caller<br/>use slot.1 to slot.3 again"]
    b1 --> b2
    b1 --> b3
    b1 --> b4
    b3 -- "on" --> b2
    b3 -- "of" --> b4
    b3 --> b5 --> b6 --> b7
```

## A call on the line

```mermaid
sequenceDiagram
    participant A as Page A
    participant X as exchange: route, hub
    participant R as rules: system on the board
    participant B as Page B
    A->>X: GET /call/APP/ROOM?peer=a
    X->>R: join::open
    R-->>X: note First to a
    X-->>A: roster, ice
    B->>X: GET /call/APP/ROOM?peer=b
    X-->>B: roster, ice
    A->>X: POST join
    X->>R: post: join::join
    R-->>X: notes Roster to a and b
    X-->>A: roster with a
    X-->>B: roster with a, the room rings
    B->>X: POST join
    X-->>A: roster with a and b
    X-->>B: roster with a and b
    A->>X: POST send to b: offer, candidates
    X->>R: post: pass::pass
    R-->>X: notes Signal to b, in order
    X-->>B: from a: offer, candidates
    B->>X: POST send to a: answer, candidates
    X-->>A: from b: answer, candidates
    A->>B: sound, camera, screen: peer to peer, not through the line
    A->>X: the page closes
    X->>R: leave::close
    R-->>X: note Roster to b
    X-->>B: roster without a
```

## One wire

```mermaid
graph TB
    w1["a wire is taken in<br/>exchange/mod.rs: take, one thread a wire"]
    w2["Ask::read<br/>the first line, the head, the body"]
    w3{"OPTIONS?"}
    w4["204, or 403 for a page that may not call"]
    w5{"is the path a room's?"}
    w6["404"]
    w7["Guest::of<br/>the cookie's caller, or a new guest and a new cookie"]
    w8{"GET or POST?"}
    w9["Hub::open: join::open<br/>refused: 400 · 403 · 429"]
    w10["the head of a stream is written"]
    w11["a second thread writes what the hub hands over<br/>each write that is taken: beat::seen"]
    w12["this thread reads until the page is gone<br/>then Hub::close: leave::close"]
    w13["Post::read, Hub::post: system::post<br/>refused: 400 · 403 · 409"]
    w14["200: who is in the call"]
    w1 --> w2 --> w3
    w3 -- yes --> w4
    w3 -- no --> w5
    w5 -- no --> w6
    w5 -- yes --> w7 --> w8
    w8 -- GET --> w9 --> w10 --> w11
    w10 --> w12
    w8 -- POST --> w13 --> w14
```

## A page that went away

```mermaid
graph TB
    g1["the page closes, or its network drops"]
    g2["the wire's read ends<br/>route.rs: stream"]
    g3["a write to the wire fails<br/>the writing thread"]
    g4["nothing is taken for longer than limit.gone<br/>pulse: beat::beat, every limit.beat"]
    g5["leave::close<br/>only if this opening is still the line's"]
    g6["the line, and its room and caller when they have nothing left, are forgotten"]
    g7["note End: the line loses its way, its thread stops"]
    g8["note Roster to every page left in the room, if it was in the call"]
    g1 --> g2 --> g5
    g1 --> g3 --> g5
    g1 --> g4 --> g5
    g5 --> g6
    g5 --> g7
    g5 --> g8
```

## The window

What it reads:

```mermaid
graph TB
    t_data["data/*.tsv<br/>built into the program"]
    phiora["Phiora<br/>asked first"]
    t_site["puikit/app/phiogy/public/data<br/>phiogy.json, room.json, host.json<br/>shop.json and a shop's own file"]
    t_keep["the person's folder<br/>setting.tsv, state.json, session"]
    t_art["puikit/app/phiegg/public<br/>pictures, by their keys"]
    p_site["Site<br/>venues, rooms, people, shops"]
    p_kept["Kept<br/>what the person wrote,<br/>marked, joined, bought"]
    p_host["Hosted<br/>a key to its file"]
    p_desk["Desk<br/>what every part works on"]
    phiora -- "GET /app/phiogy" --> p_site
    t_site -- "when Phiora does not answer;<br/>rooms, hosts and shops always" --> p_site
    phiora -- "/me, /state/NAME" --- p_kept
    t_keep --> p_kept
    t_data --> p_desk
    p_site --> p_desk
    p_kept --> p_desk
    t_art --> p_host
```

What draws it:

```mermaid
graph TB
    p_app["Phiogy: PApp<br/>reads the tables"]
    p_desk["Desk"]
    board["Board<br/>behind one lock,<br/>changed by the line"]
    p_room["a room<br/>title, stream, present,<br/>person, viewer"]
    p_around["around the rooms<br/>guide, search, venue, friend,<br/>people, shop, account"]
    p_fact["fact.rs"]
    k_stage["appkit: PStage<br/>lays out, draws, sends"]
    k_bar["appkit: the bar, its menu,<br/>the profile, a modal's panel"]
    k_screen["appkit: screen<br/>the window, through gpui"]
    p_host["Hosted"]
    p_app --> p_desk
    p_app --> k_stage
    p_desk --> p_room
    p_desk --> p_around
    p_desk --> p_fact
    board --> p_room
    board --> p_fact
    k_stage --> p_room
    k_stage --> p_around
    k_stage --> k_bar
    p_fact --> k_bar
    k_stage --> k_screen
    p_host --> k_screen
```

## The window as a caller

```mermaid
graph TB
    subgraph win["the window"]
        j_pill["Title, Join: part/title.rs, join.rs<br/>Call, Call with the camera;<br/>or who is calling, and Join"]
        j_call["Calling: part/call.rs<br/>the tiles; voice, camera, screen, end,<br/>each on its ground in the group's"]
        j_voice["Voice: part/voice.rs<br/>who is in a voice room;<br/>Join, Leave, Mute, Sound off"]
        j_foot["Foot: part/guide/foot.rs<br/>the voice room I am in,<br/>at the foot of the menu on every page"]
        j_desk["Desk<br/>dial · join · leave · hush · deaf<br/>camera · share · listen"]
        j_present["Present<br/>this window, and of each other caller:<br/>how far, muted, pictures, how loud"]
    end
    subgraph peer["peer/: on a thread of its own"]
        p_call["Call<br/>asked: hush, deaf, camera, share, end<br/>read: Seen"]
        p_ring["Ring<br/>a room's line listened to<br/>from outside its call"]
        p_line["Line<br/>the room's line as a page holds it"]
        p_mesh["Mesh<br/>one bond a caller: polite, open, ignore"]
        p_engine["Engine, Link<br/>offer · accept · candidate · take"]
    end
    subgraph carry["what carries it"]
        c_stand["Stand<br/>agrees, carries nothing"]
        c_rtc["rtc: libwebrtc<br/>feature rtc"]
    end
    line["the line: exchange/<br/>GET stream · POST join, send"]
    page["a browser's page<br/>call.ts, peer.ts"]
    j_pill --> j_desk --> p_call
    j_call --> j_desk
    j_voice --> j_desk
    j_foot --> j_desk
    j_desk --> p_ring --> p_line
    p_call --> j_present
    p_call --> p_line
    p_call --> p_mesh --> p_engine
    p_engine --> c_stand
    p_engine --> c_rtc
    p_line -- "HTTP, as a page" --> line
    page --> line
    c_rtc -- "sound and picture, peer to peer" --> page
```

What the real library carries, and where it comes from and goes:

```mermaid
graph LR
    subgraph device["where it comes from: data/peer.tsv, device"]
        d_system["system<br/>mic · camera · scene"]
        d_stand["stand<br/>tone: a tone, a grey ramp"]
    end
    subgraph rtc["peer/rtc/"]
        c_sent["sent, wire<br/>sound, camera, screen<br/>on every link"]
        c_ear["ear<br/>measures what arrives"]
    end
    far["another caller"]
    speaker["speaker<br/>the system's sound out"]
    film["appkit: PFilm<br/>the pictures that move"]
    tile["Calling: part/call.rs<br/>the tiles"]
    d_system --> c_sent
    d_stand --> c_sent
    c_sent -- "peer to peer" --> far
    far -- "peer to peer" --> c_ear
    c_ear --> speaker
    c_ear --> film --> tile
    d_system -- "its own picture" --> film
```

## Who gives way

```mermaid
sequenceDiagram
    participant R as this window: phiogy-...
    participant L as the line
    participant P as a page: a uuid, the lesser name
    R->>L: POST join
    L-->>P: roster with the window, the room rings
    P->>L: POST join
    L-->>R: roster with both
    L-->>P: roster with both
    Note over R,P: both come with something to send, so both offer at once
    R->>L: send: state, offer
    P->>L: send: state, offer
    L-->>P: the window's offer
    L-->>R: the page's offer
    Note over R: the greater name: its own offer is out, the page's is let pass
    Note over P: the lesser name: it takes its own offer back and answers
    P->>L: send: answer, candidates
    L-->>R: answer, candidates
    R->>L: send: candidates
    L-->>P: candidates
    R->>P: a tone and a grey ramp
    P->>R: its microphone, its camera
    Note over R,P: what changes later, a camera turned on, is offered and answered the same way
```

## Data: one source, three words

What a person makes and others read (a club, a host, a message, a request,
a like) is a **record** on Phiora, and the window asks for it by **kind**
and **name key** with a **payload**. Nothing in a part knows a path, a
header or a table: the part says what it wants to the desk, the desk says
it to `Records`, and one function answers.

```mermaid
flowchart LR
  subgraph window["The window (parts)"]
    room["room: say, like, reply"]
    hosts["hosts: make, publish, join"]
    friends["friends: ask, accept, remove"]
    draw["drawers: publish a club, a PFT"]
  end
  desk["Desk (part/*.rs)\nsay, give, follow, bind,\ncreate_with, adopt, publish_host"]
  subgraph one["records.rs: one match"]
    ask["Ask: Has | Get | Set(payload)"]
    act["Records::act(kind, key, ask)"]
    reply["Reply: Has(bool) | Got(record) | Done(Made | Changed | Same)"]
  end
  kept["Kept (kept.rs)\nthe person's documents, who they are,\nsend() in order on a thread,\nheard() what others said"]
  verb["phiora.rs\nVerb: Get | Put | Post | Patch | Delete"]
  server[("Phiora\n/tree (where, limit), /tree/item,\n/state, /auth, /me")]
  room --> desk
  hosts --> desk
  friends --> desk
  draw --> desk
  desk -->|has, get, set| act
  ask --> act --> reply
  desk -->|send, heard| kept
  act --> verb
  kept --> verb
  verb --> server
```

The three words and what they do:

| Word | Asks | Answers | Used for |
|---|---|---|---|
| `has(kind, key)` | does Phiora have it | true or false (not knowing is false) | a host that is already there |
| `get(kind, key)` | the record | its payload with its id, or none | reading one record |
| `set(kind, key, payload)` | make it, or change it to match | `Made(id)`, `Changed(fields)` or `Same` | every write |

`set` sends only the fields that differ, so the same payload again is
`Same` and nothing is sent. Who may change a record is Phiora's to say (its
maker, the person it names in `userId`, an editor); a guest is refused, and
`Kept::records()` says so before it asks. Where what is said or liked must
not hold the window up, `Kept::send(kind, key, payload)` does the same `set`
on the thread that keeps the person's changes in order.

The kinds now, each by its name key:

| Kind | Name key | Payload | Written by | Read by |
|---|---|---|---|---|
| `club` | `club_<name>` | name, ticker, supply, tagline | the publish drawer, `--make` | Phiegg's web |
| `profilePft` | `pft_<club>_<number>` | name, number, `clubId`, traits | the publish drawer, `--make` | Phiegg's web |
| `host` | the host's id (`m...`) | name, genre, icon, rooms | Create a host | the Hosts page of others (Join) |
| `message` | the message's id (`m<moment>`) | room, text, reply, author's name | Send | everyone in the room, every 4 seconds |
| `bond` | `<from>><to>` | from, to, `userId` (the one asked), state | Add friend, Accept, Remove | both of them |
| `mark` | `mk:<me>:<room>:<message>:<icon>` | room, message, icon, on or off | Like | everyone in the room |
| `follow` | `fl:<me>><them>` | from, to, on or off | Follow | nobody yet |

What the window asks of Phiora is narrowed there: `GET /tree?kind=...&where=
field:value&limit=N` returns the last N rows that match (a room, the
requests that name the person, one record by its key), not every record.

Where the window finds Phiora, today: the place is `--phiora`, else the
system's `PHI_API`, else the app's own row (`data/app.tsv`, `server.phiora`);
who is asking is the session Phiora gives a person (`/me`), kept in their
own folder. The records code (`records.rs`, `phiora.rs`, `net.rs`) is in
Phiogy; the wire (`Verb`) and the three words are not Phiogy's alone, and
want to move into appkit for every app to use. That is **not built**: see
the next section.

## One data source for every app: not built

```mermaid
flowchart LR
  app["Any app (Phiogy, Phigen, ...)"]
  src["appkit: a data source\nhas, get, set (the three words)\nand one place and key to ask with"]
  cfg["Where and with what key:\nthe app's own row (data/app.tsv)\nor the system's, for every app"]
  phiora[("Phiora")]
  app -->|has, get, set| src
  cfg --> src
  src --> phiora
```

An app would say only `has`, `get` and `set` on a kind and a key. The place
of Phiora and the key an app asks with would be set in the app's own table
or once for all the apps; the app never sees a path, a verb or a header.
Phiora's server (`puikit/app/phiora-server`) already narrows what it is
asked (`where`, `limit`); what a record may hold, and who may change it, is
its own to keep. Today nothing in Phiogy has a key of its own: a person is
who Phiora says their session is, and the call line shares one word with
Phiora (`PHI_CALL_KEY`).

## What is next: not built

```mermaid
graph LR
    line["the line, and the window as a caller<br/>built"]
    peer["a real microphone and camera<br/>what is heard played, what is seen shown"]
    msg["messages and requests<br/>reach the other person"]
    turn["a relay: TURN<br/>two networks can meet"]
    sfu["a forwarding unit<br/>rooms above 8"]
    cloud["the route taken over<br/>on the server in the cloud"]
    who["who a caller is<br/>asked of Phiora, not a guest's cookie"]
    line -.-> msg -.-> turn -.-> sfu -.-> peer
    line -.-> cloud
    cloud -.-> who
    style peer stroke-dasharray: 5 5
    style msg stroke-dasharray: 5 5
    style turn stroke-dasharray: 5 5
    style sfu stroke-dasharray: 5 5
    style cloud stroke-dasharray: 5 5
    style who stroke-dasharray: 5 5
```

## Tasks, planned

Not built: asked for on 23 November 2026, as Microsoft To Do stands in
the user's picture. A task is a person's own, or a host's; it can come
from a message and be given to somebody in the host.

```mermaid
graph TB
    subgraph data["data/"]
        t_list["task.tsv<br/>the lists: my day, important, planned,<br/>assigned, flagged, all; icon, words, rule"]
        t_keep["keep.tsv<br/>doc.task: the person's own tasks"]
    end
    subgraph rule["no window in these"]
        t_comp["comp/task.rs<br/>Task: what, done, important, due,<br/>repeat, steps, note, given to, from, host"]
        t_system["system/task.rs<br/>add, finish, star, plan, give;<br/>which list a task stands in"]
    end
    subgraph win["the window"]
        t_menu["the bar's menu<br/>the lists, each with how many"]
        t_page["part/task.rs<br/>grid or list, add a task,<br/>sort and group"]
        t_one["one task, beside them<br/>steps, the day, a note, who has it"]
        t_stream["a message's controls<br/>flag: make a task of it"]
    end
    subgraph far["shared"]
        t_phiora["Phiora<br/>a host's list, for everyone in it"]
    end
    t_list --> t_system
    t_comp --> t_system
    t_system --> t_page
    t_menu --> t_page
    t_page --> t_one
    t_stream --> t_system
    t_system --> t_keep
    t_system --> t_phiora
```
