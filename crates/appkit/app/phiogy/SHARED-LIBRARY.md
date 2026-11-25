# Phiogy online: domains, calls between people, and a shared asset library

What it takes for Phiogy to run on the server, for team members to call
each other, and for the asset library to be uploaded to and shared from
the server instead of Google Drive. Written 25 November 2026 from what is
on the server and on the Drive now. Nothing here is built yet.

## 1. Domains

| Domain | DNS | Points at | Served by Caddy |
|---|---|---|---|
| phiegg.com, www | GoDaddy (ns63/64.domaincontrol.com) | 13.63.125.132 | yes, plus `/cdn/` |
| phiace.ai, www | GoDaddy (ns73/74.domaincontrol.com) | 13.63.125.132 | yes |
| phiber.ai, www | | 13.63.125.132 | yes |
| phipad.xyz, www | | 13.63.125.132 | yes |
| **ogy.phiegg.com** | **no record** | - | no |
| **app.phiace.ai** | **no record** | - | no |

Today the web Phiogy is at `phiegg.com/phiogy/`, Phiora's API at
`phiegg.com/api/`, pictures at `phiegg.com/cdn/<key>`.

**Recommended: `ogy.phiegg.com`.** It is the same site as phiegg.com, so
the Phiora session, the API and the CDN are all one origin family; no
cross-site cookies. `app.phiace.ai` can be added beside it and redirect,
or later become the team's workspace.

To turn either on:
1. **You, at GoDaddy:** an `A` record `ogy` → `13.63.125.132` on
   phiegg.com (and/or `app` → the same on phiace.ai).
2. **Me, on the server:** a Caddy site `ogy.phiegg.com { import cdn;
   import app }` with the root going to `/phiogy/`; Caddy gets the
   certificate itself. nginx already serves `/phiogy/`, `/api/` and the
   line.

## 2. Calls between people (inter-calling)

The `phiogy` service (the Rust line, image `phiogy-line`) is already up on
the server beside Phiora, and coturn runs on the host. What blocks calls
between two people on different networks:

1. **You, in the AWS console:** the security group lets in only 22, 80 and
   443. Open **UDP+TCP 3478** and **UDP 49152-49407** (the relay).
2. **Me:** point the desktop Phiogy's line at the server
   (`wss://ogy.phiegg.com/line`) instead of `localhost:4310`, with the
   relay's address and word from `.env` (`PHI_CALL_ICE`); keep the local
   line for working offline.
3. **Test:** two windows on two networks (one on a phone hotspot) call
   each other; FLOW.md gets the steps.

## 3. The trait sheets found on the Drive

| Sheet (Drive) | What it holds | Rows |
|---|---|---|
| **BPWC-1-1000** (tabs `Hair 01`-`Hair 05`) | Bold Pups: Number, Background, Char, Hair, Fur, Hair Colour, Outfit, Glasses, Hat, Expression, Club | 5 × 2,000 = **10,000** (BPSC 1-8,000, HHOC 8,001-10,000) |
| **FINAL-NFT-NONPIXEL** (`EPAC:NON-PIXEL`, `EPAC:PIXEL`) | Epic Apes: Number, Rarity (background), Char, Hair, Hat, Accessory, Hair Colour | 1,024 each |
| **Blod Fox 1-1000** | Number, Background, Outfit, Hats and Glasses, Fur Colour | 1,000 numbers, **traits not filled in yet** |
| **BAEC.xlsx** (shared by Drijke) | the trait vocabulary: rarities with counts, roles, hairdos, expressions, eye wear, colours | the allowed values |
| BOLD_PUPS (sheet and .xlsx) | the production tracker (hours, people), not traits | - |

Phiora today holds only 711 Bold Pups (`profilePft`, from Phiegg's
`assets.json`) with two traits each (background, outfit). The sheets
have all 10,000 with eleven.

## 4. How files are kept, uploaded and synced

**Files.** Every file is kept once on the server, named by its contents:
`/srv/phi-cdn/lib/<sha256>.<ext>`, served as
`phiegg.com/cdn/lib/<sha256>.<ext>` (Caddy already serves `/srv/phi-cdn`).
A file never changes under its name, so it can be cached forever and two
people uploading the same file store it once. The disk has 40 GB free; the
10,000 pups at about 0.5 MB each are about 5 GB. Past that, the same keys
move to S3 behind the same URL.

**Records.** Phiora keeps what a file is, in its one `items` table:
- `collection`: BPSC, HHOC, EPAC, Blod Fox; its traits and each trait's
  allowed values (from BAEC.xlsx and the sheets).
- `profilePft` (one per token): number, collection, club, every trait,
  its picture's key.
- `asset`: any other file in the library (a model, a look, a render):
  name, key, size, type, the collection or folder it is in, who sent it,
  and its state: **draft → in review → approved**. Only approved assets
  show in Phiegg; this replaces "approved looks are the ones on the Drive".

**Upload (new).** Phiora gets `POST /file`: signed-in team members only,
a size limit and a list of allowed types; it streams the file to the CDN
folder under its hash and answers the key. Then `POST /tree/item` makes
the `asset` record. In Phiogy: drag files onto a library place, or the
library's Import, and choose "send to the team".

**Sync (new).** Phiogy's library gets a **Team** place that lists the
`asset` and `collection` records from Phiora. A file is fetched only when
opened, into a local cache folder by its key, and checked against its
hash. Phiogy asks Phiora's change log (`/tree/change`, which needs an
`after=<seq>` so it gives only what is new) every so often, so a file
sent, approved or removed by anyone shows for everyone. Records are
last-writer-wins with the change log as history; files never conflict.

**Who may do what.** Team members are Phiora accounts named in
`PHI_EDITORS` (empty on the server today) or a `team` role; approvers a
smaller list. Guests read approved assets only.

## 5. Uploading a collection, step by step (Bold Pups)

1. **Read the traits** from BPWC-1-1000, all five tabs, through the
   Sheets connector (no download): 10,000 rows.
2. **Check them**: numbers 1-10,000 once each; club BPSC for 1-8,000 and
   HHOC after; every value in the vocabulary (BAEC.xlsx); report blanks
   and strays before anything is written.
3. **Write the collection**: BPSC and HHOC as `collection` records, each
   trait with its allowed values and counts (rarity from how often a value
   occurs).
4. **Write the tokens**: 10,000 `profilePft` records with stable ids
   (`pft_bpsc_0001` → Phiora's md5 of `profilePft:pft_bpsc_0001`),
   replacing the 711 partial ones, through an import script against
   Phiora (`npm run import -- bpwc`), dry run first.
5. **Bring the pictures**: the finals are in the Drive's FINAL-NFTS
   (BPSC, HHOC). The connector cannot copy folders, so they come down as
   zips (**needs your yes: what, from where, how large**), are named by
   number, hashed, and sent with `scripts/media/sync.sh`; each token gets
   its picture's key.
6. **See it**: Phiegg's BPSC page and Phiogy's library place `BPSC` show
   the 8,000 as a grid with a filter per trait, drawn from the
   collection's trait list, not written per collection.
7. **Then the others**: EPAC the same way (1,024 + 1,024 pixel); Blod Fox
   waits for its traits.

## 6. What only you can do

- The GoDaddy record(s) for `ogy.phiegg.com` / `app.phiace.ai`.
- Open the relay ports in the AWS security group.
- Name the team members (Phiora handles) and the approvers.
- Say yes to downloading the FINAL-NFTS zips, and to re-seeding Phiora
  (it still has the old seed; some people, such as Judy DuBois, are not in
  it).

## 7. Order of work

1. Domain live (after your record) and calls between two networks.
2. `POST /file`, the `asset` record and the Team place in the library.
3. The Bold Pups import (steps 5.1-5.4), then the pictures (5.5).
4. Review and approval in the library; Phiegg shows approved only.
5. EPAC, then the rest of the Drive's finals.
