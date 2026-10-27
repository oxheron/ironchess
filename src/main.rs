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

fn main() -> std::io::Result<()> {
    uci::run()
}
