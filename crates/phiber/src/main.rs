#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use phiber::app;
use phiber::v2::cli::render::{BHeadlessRender, BLookOverride};
use phiber::v2::cli::{BCli, help};

fn main() {
    // Install a panic hook that prints the panic location + payload before
    // the app dies, so crashes are diagnosable instead of silent aborts.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("========== PANIC ==========");
        eprintln!(
            "Location: {}",
            info.location()
                .map(|l| l.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
        let payload = info.payload();
        if let Some(s) = payload.downcast_ref::<&str>() {
            eprintln!("Payload: {}", s);
        } else if let Some(s) = payload.downcast_ref::<String>() {
            eprintln!("Payload: {}", s);
        } else {
            eprintln!("Payload: (non-string panic)");
        }
        eprintln!("===========================");
    }));

    // The command line, as Blender takes it (`v2::cli`). Everything the
    // app itself does is unchanged when no flag asks otherwise, so running
    // the binary with no arguments opens the window exactly as before.
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match BCli::parse(&argv) {
        Ok(cli) => {
            if cli.help {
                print!("{}", help::help());
                return;
            }
            if cli.version {
                println!("{}", help::version());
                return;
            }
            if cli.engine.as_deref() == Some("HELP") {
                print!("{}", help::engine_list());
                return;
            }
            if cli.format.as_deref() == Some("HELP") {
                print!("{}", help::format_list());
                return;
            }
            // `-b … -f` / `-b … -a`: render and exit, through the same
            // `BHeadlessRender` the examples run. Without a render flag a
            // file argument opens that file in the window, as Blender does.
            if cli.renders() {
                std::process::exit(render(&cli));
            }
            app::run(cli.file.clone());
        }
        Err(error) => {
            eprintln!("{}", error.message());
            std::process::exit(1);
        }
    }
}

/// `-b … -f/-a`: open the file, resolve the frames against its own range,
/// and render them. Returns the process status.
fn render(cli: &BCli) -> i32 {
    let Some(file) = cli.file.clone() else {
        eprintln!("Error: no blend loaded. cannot use '-f / --render-frame'");
        return 1;
    };
    let model = match phiber::phiphx::Phx::open(std::path::Path::new(&file)) {
        Ok(model) => model,
        Err(why) => {
            eprintln!("Error: cannot read '{file}': {why:?}");
            return 1;
        }
    };
    // The relative forms in `-f`, `-s` and `-e` are relative to the file's
    // own range, so the frames cannot be resolved until it is open.
    let scene = phiber::v2::cli::frame::BFrameLimit {
        first: model.timeline.frame_start,
        last: model.timeline.frame_end,
    };
    let frame = match cli.frame_list(scene) {
        Ok(frame) => frame,
        Err(why) => {
            eprintln!("{}", why.message());
            return 1;
        }
    };
    if frame.is_empty() {
        eprintln!("Warning: no frames to render");
        return 0;
    }
    let out = cli.output.clone().unwrap_or_else(|| "//render/".to_string());
    // `//` is relative to the blend file, as Blender resolves it.
    let out = match out.strip_prefix("//") {
        Some(rest) => std::path::Path::new(&file).parent().unwrap_or(std::path::Path::new(".")).join(rest).display().to_string(),
        None => out,
    };
    let render = BHeadlessRender {
        file,
        out,
        frame: frame.iter().map(|f| *f as f32).collect(),
        samples: cli.samples,
        draft: cli.draft,
        camera: cli.camera.clone(),
        look: BLookOverride::from_env(),
    };
    if !cli.quiet {
        println!(
            "Rendering {} frame(s) on {} at {} samples -> {}",
            render.frame.len(),
            cli.engine_name(),
            render.sample_count(),
            render.out
        );
    }
    match render.run() {
        Ok(cost) => {
            if !cli.quiet {
                for line in cost.iter().map(|c| c.line()) {
                    print!("{line}");
                }
            }
            0
        }
        Err(why) => {
            eprintln!("Error: {why}");
            1
        }
    }
}
