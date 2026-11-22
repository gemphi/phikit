//! A caller with no window: what `phiogy --caller` does, and what a
//! window asked to say each second what its call sees prints.

use appkit::service::PTable;

use super::given;

/// The rows of `data/peer.tsv` given another value for this run, each
/// as `--set NAME=VALUE`: `--set device=stand` has a call send the
/// tone and the grey ramp in the microphone's and the camera's place.
pub(super) fn over(word: &[String]) -> Vec<(String, String)> {
    given(word, "--set").iter().filter_map(|pair| pair.split_once('=')).map(|(name, value)| (name.to_string(), value.to_string())).collect()
}

/// What makes the engine that carries a call: the real library where
/// it is built in, its pictures shown on that film; the stand-in
/// where it is not.
#[cfg_attr(not(all(feature = "rtc", not(target_family = "wasm"))), allow(unused_variables))]
pub(super) fn engine(over: &[(String, String)], film: &appkit::pui::core::PFilm) -> crate::peer::Maker {
    #[cfg(all(feature = "rtc", not(target_family = "wasm")))]
    return crate::peer::rtc::maker(over.to_vec(), film.clone());
    #[cfg(not(all(feature = "rtc", not(target_family = "wasm"))))]
    Box::new(|| Box::new(crate::peer::Stand::default()))
}

/// What the window is asked to do in a call from the start: `--join`
/// joins the open room's, `--camera` turns the camera on in it and
/// `--share` shares a screen.
pub(super) fn begin(desk: &crate::part::Desk, word: &[String]) {
    let has = |flag: &str| word.iter().any(|word| word == flag);
    if has("--join") {
        desk.join();
        desk.camera(has("--camera"));
        desk.share(has("--share"));
    }
}

/// What a call has seen, as lines to print: whether this caller is in
/// the call, and of each other caller how far, muted, how loud, and
/// the pictures that came.
pub(super) fn said(seen: &crate::peer::Seen, text: &PTable) -> Vec<String> {
    let head = text.fill("caller.seen", "count", &seen.far.len().to_string()).replace("{joined}", &seen.joined.to_string()).replace("{engine}", &seen.engine);
    let own = text.fill("caller.own", "camera", &seen.own.camera.to_string()).replace("{screen}", &seen.own.screen.to_string()).replace("{level}", &format!("{:.3}", seen.own.level)).replace("{fault}", &seen.own.fault.iter().zip(&seen.own.why).map(|(name, why)| format!("{name} ({why})")).collect::<Vec<_>>().join(", "));
    let far = seen.far.iter().map(|far| {
        let line = text.fill("caller.far", "peer", &far.peer).replace("{stage}", &format!("{:?}", far.stage)).replace("{quiet}", &far.quiet.to_string());
        line.replace("{level}", &format!("{:.3}", far.level)).replace("{count}", &far.seen.to_string()).replace("{width}", &far.size.0.to_string()).replace("{height}", &far.size.1.to_string())
    });
    [head, own].into_iter().chain(far).collect()
}

/// A caller with no window: join a room's call and say each second
/// what is heard and seen of the others. A line on its standard input
/// has it mute, unmute or leave: the words are `data/text.tsv`'s. It
/// ends when it is told to leave, when its time is up, when the call is
/// over, or saying why it could not begin.
pub(super) fn caller(word: &[String], key: &str, own: &Result<String, String>, text: &PTable) {
    use crate::peer::{Call, name};
    let place = given(word, "--at").pop().or_else(|| own.clone().ok()).unwrap_or_default();
    let time = given(word, "--for").pop().and_then(|time| time.parse::<u64>().ok());
    let call = name().ok_or_else(String::new).and_then(|name| Call::start(&place, key, &name, engine(&over(word), &appkit::pui::core::PFilm::default())).map_err(|error| error.to_string()));
    let call = match call {
        Ok(call) => call,
        Err(why) => return eprintln!("{}", text.fill("caller.fail", "why", &why)),
    };
    println!("{}", text.fill("caller.begin", "peer", &call.peer).replace("{key}", key).replace("{at}", &place));
    call.camera(word.iter().any(|word| word == "--camera"));
    call.share(word.iter().any(|word| word == "--share"));
    let (tell, told) = std::sync::mpsc::channel::<String>();
    _ = std::thread::Builder::new().spawn(move || std::io::stdin().lines().map_while(Result::ok).for_each(|line| _ = tell.send(line.trim().to_string())));
    for _ in 0..time.unwrap_or(u64::MAX) {
        std::thread::sleep(std::time::Duration::from_secs(1));
        for asked in told.try_iter() {
            match asked.as_str() {
                asked if asked == text.get("caller.mute") => call.hush(true),
                asked if asked == text.get("caller.unmute") => call.hush(false),
                asked if asked == text.get("caller.camera") => call.camera(true),
                asked if asked == text.get("caller.still") => call.camera(false),
                asked if asked == text.get("caller.share") => call.share(true),
                asked if asked == text.get("caller.unshare") => call.share(false),
                asked if asked == text.get("caller.leave") => return,
                _ => {}
            }
        }
        let seen = call.seen();
        said(&seen, text).iter().for_each(|line| println!("{line}"));
        if seen.over.is_some() {
            return eprintln!("{}", text.fill("caller.fail", "why", &seen.over.unwrap_or_default()));
        }
    }
}
