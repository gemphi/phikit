//! Open Phigen in a window at a desk. What the command line can say is in
//! `run.rs`; in a browser Phigen starts from `web.rs`.

fn main() {
    let word: Vec<String> = std::env::args().skip(1).collect();
    phigen::run(&word);
}
