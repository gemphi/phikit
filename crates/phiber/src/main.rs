#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use phiber::app;

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

    app::run();
}
