# phicore — support crates (vendored)

Nothing here is UI. These are the utilities [`../phigpu`](../phigpu) and
[`../phiui`](../phiui) are built from; phiber never names them itself, it gets
them through those two.

| Crate | Reached through | What it does |
|---|---|---|
| `collections` | `gpui`, `theme` | the map and set types they use |
| `sum_tree` | `gpui` | the balanced tree behind long lists and text |
| `scheduler` | `gpui` | runs async tasks on the UI thread |
| `http_client` | `gpui` | fetching over HTTP |
| `util`, `path` | `http_client` | string and path helpers |
| `media` | `gpui` | image and video frame types |
| `refineable`, `util_macros` | `theme`, `util` | derive macros |
| `zlog`, `ztracing`, `ztracing_macro` | `gpui`, `sum_tree` | logging and tracing |

Remove any one and phiber stops building — check with
`cargo tree -p phiber` before touching them.
