# philog

The session journal phikit's crates log through: **one file per run**, beside
the executable, with every `log` record at its time, the gap since the last
line, and the timings a caller marks.

```
    0.000s  +     0.0ms  Phiber session journal — release build, C:\...\phiber.exe
    0.412s  +   412.3ms  [INFO] wgpu_hal::vulkan::instance: Vulkan API 1.3
    1.044s  +   631.9ms  Opening Barbershop.blend
   13.287s  + 12243.0ms  Opened Barbershop.blend: 4,281 objects
   13.301s  +    14.1ms  Reading Barbershop.blend took 12.243s
   19.884s  +  6583.0ms  slow frame   61.4ms | tree   8.1 | draw   53.3 | ...
```

It is a **sink for the `log` facade**, not a logging API. A crate that
already writes `log::info!` is journalled by installing it; no call site
changes.

```rust
philog::install();                            // the `log` facade's records
philog::arm("simkit-session.log");            // beside the executable
philog::note(philog::header("SimKit session journal"));
philog::note_panics();                        // a crash says why, on disk

log::info!("RPC Server listening on 0.0.0.0:41451");
let _span = philog::BSpan::open("Flight");    // "Flight took 126.312s"
```

## What it promises

- **Nothing unarmed.** Until a run calls `arm`/`arm_at`, every note and every
  record is dropped: tests, benchmarks and libraries linked into someone
  else's binary leave no file.
- **No app required.** The journal is a process-wide static, so a library with
  no app around it — simkit's RPC server — arms it and writes.
- **Readable after a crash.** Lines go straight to the file with no buffer in
  the way, `flush` runs on a schedule, the journal flushes again as it closes,
  and a panic on any thread writes its reason before the process's own hook.

## The pieces

| module | what it is |
| --- | --- |
| `journal` | the file, its lines, `arm`/`note`/`flush`, the panic hook, and `Feed` for a caller draining its own timestamped records under one lock |
| `sink` | the `log` facade's way in, at Info and above by default |
| `span` | `BSpan` and `timed`: how long a piece of work took, beside the lines it took it in |

## Who arms it

- **phiber** — `app::journal::BJournal`: names the file `phiber-session.log`,
  echoes notes to the console under `PHIBER_NOTES`, and feeds the in-app Info
  log (`state::log`) to the journal each frame.
- **simkit** — `server::journal`: `simkit-session.log`, armed by
  `RpcServer::start` and nothing else, with a span per client connection and a
  line for every request over `SLOW_MS`.

Not to be confused with `phicore/zlog`, the structured logger inherited from
Zed that `sum_tree` and `ztracing` use. philog is the session record; zlog is
Zed's logging stack, and the two are left alone to do their own jobs.
