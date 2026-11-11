# Jobs — `phiphx/blend/file`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 1 | 4 |

## Todo

- [ ] Memory-mapped / lazy block loading for files over 1 GB

## Done

- [x] v0 and v1 headers, `LargeBHead8`, zstd / gzip
- [x] Old-address index plus per-ID resolution of repeated addresses (`scope`)
- [x] DNA layout incl. pointer arrays; every struct size checked against the file
- [x] Quick peek (`REND`, `TEST` thumbnail) without parsing

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
