//! Open the botanist in a window at a desk. What the command line can
//! say is in `run.rs`.

fn main() {
    let word: Vec<String> = std::env::args().skip(1).collect();
    phibot::run(&word);
}
