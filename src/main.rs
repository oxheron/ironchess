mod eval;
mod game;
mod movegen;
mod search;
mod uci;
mod utils;
mod zobrist;

#[cfg(test)]
#[path = "../test/perft.rs"]
mod perft;

#[cfg(test)]
#[path = "../test/support.rs"]
mod test_support;

fn main() -> std::io::Result<()> {
    uci::run()
}
