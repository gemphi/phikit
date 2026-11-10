# connect

How the world reaches the database, the session journal and data files,
planned from what exists (read 2026-11-10, nothing run). Solid boxes are
in the code today; dashed boxes are to build.

```mermaid
graph LR
    w_world["phiecs World"]
    w_rec["system/time/record<br/>Readout: each changed timed aspect,<br/>each event, as a row"]
    w_sink["Sink<br/>the one way out"]
    s_mem["memory journal<br/>in phiecs"]
    s_log["session file<br/>philog: install · arm · note<br/>one readable file a run"]
    s_db["database<br/>phiora, through its API"]
    w_world --> w_rec --> w_sink
    w_sink --> s_mem
    w_sink --> s_log
    w_sink --> s_db
    subgraph phiora["phiadk/phiora: FastAPI under /api/v1, Postgres"]
        t_kinds["kinds<br/>name · app · label · spec"]
        t_items["items<br/>id · kind · slug · doc"]
        t_links["links<br/>from_id · field · to_id · data"]
        t_meta["meta<br/>item_id · key · value"]
        t_log["log<br/>seq · at · action · kind · item_id · data"]
    end
    s_db --> t_kinds
    s_db --> t_items
    s_db --> t_links
    s_db --> t_log
    classDef plan stroke-dasharray: 6 4;
    class w_rec,w_sink,s_mem,s_db plan;
```

## What maps to what

The right-hand columns are tables that exist today in the Python service
(`phiadk/phiora/alembic/versions/0001_core.py:15-109`).

| In the world | In phiora | Rule |
|---|---|---|
| `Kind` | a row of `kinds`: `name`, `app = phikit`, `spec` | declared once, in one data file; the same list goes to `World` at start and to the database. `spec` lists the aspects a kind carries and their fields |
| a `Word` | a row of `items`: `id`, `kind`, `slug`, `doc` | `id = md5(kind:name)`, the rule already in `src/services/ids.py:9`; `slug` is the spelling |
| a word's stored aspects | `items.doc`, one object an aspect: `{ aspect name: { field: value } }` | only aspects that are `Stored` and not `Transient` |
| the `Tree` | `links`: `field = parent` | one row a child |
| a relation edge | `links`: `field` = the relation's name, `data` = the weight | the name, never the channel number: the two phiecs number relations differently |
| a journal row: an aspect's value at a tick | `log`: `action = set`, `kind`, `item_id`, `data = { tick, aspect, fields }` | appended, never rewritten |
| an event | `log`: `action` = the event's name | same |
| the clock | `meta` on the world's own item: `tick`, `time`, `fps` | - |
| an `Asset { kind, key }` | the `id` of another item, held in `doc` as a field ending `Id` | the reference convention phiora already rebuilds `links` from (`src/services/seed.py:208-227`) |

## What exists, and what is missing

```mermaid
graph LR
    subgraph have["exists today"]
        h_py["phiadk/phiora: kinds, items, links, meta, log<br/>GET kinds · GET items · POST ora/query · GET tree<br/>read-only: writes only through the seed"]
        h_node["puikit/app/phiora-server: items(kind, id, slug, doc, ...)<br/>item_changes · POST, PATCH, DELETE tree/item<br/>its own schema, not the same tables"]
        h_rs["phinet/crates/phiora: Transport trait,<br/>kinds · describe · query<br/>read-only, no HTTP transport, no crate uses it"]
        h_log["phikit/crates/philog: the session file<br/>used by phiber and simkit"]
        h_lex["phinet/crates/phistd: lexfile<br/>read · cached · append, one entry a line"]
        h_row["phinet phiecs: Row, tell, from_rows<br/>edges only, as TSV"]
    end
    subgraph gap["to build or decide"]
        g_write["write routes in the Python service:<br/>kinds, items, links, log in batches"]
        g_which["one store for the world: the Python schema has kinds,<br/>links and log, which the mapping needs"]
        g_client["the Rust client gains an HTTP transport and writes"]
        g_path["phikit cannot take a phinet path: the client and lexfile<br/>move where both repos can take them, or the sink lives in the app"]
        g_name["two crates are named philog: the session journal in phikit,<br/>a reasoning library in phinet"]
        g_stored["phikit's phiecs has no Row, Stored, dump or load yet"]
    end
    h_py --> g_write
    h_node --> g_which
    h_rs --> g_client --> g_path
    h_lex --> g_path
    h_log --> g_name
    h_row --> g_stored
    classDef plan stroke-dasharray: 6 4;
    class g_write,g_which,g_client,g_path,g_name,g_stored plan;
```

## From the first commit

```mermaid
graph LR
    d_kinds["data/kind.txt<br/>one kind a line, the lexfile format"]
    d_app["the app reads it<br/>phiecs reads no files"]
    d_world["World: kinds declared"]
    d_plug["each Plugin adds its kinds, aspects and roles"]
    d_sinks["sinks attached: memory, philog, phiora"]
    d_step["every step from the first:<br/>rows and events go to all three"]
    d_check["a kind the database does not know is refused at start,<br/>not discovered later"]
    d_kinds --> d_app --> d_world --> d_plug --> d_sinks --> d_step
    d_world --> d_check
    classDef plan stroke-dasharray: 6 4;
    class d_kinds,d_app,d_world,d_plug,d_sinks,d_step,d_check plan;
```
