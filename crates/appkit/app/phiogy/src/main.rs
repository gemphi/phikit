//! Open Phiogy in a window at a desk, or serve its line alone. What the
//! command line can say is in `run.rs`; in a browser Phiogy starts from
//! `web.rs`.

fn main() {
    let word: Vec<String> = std::env::args().skip(1).collect();
    phiogy::run(&word);
}
