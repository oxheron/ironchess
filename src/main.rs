mod game;
mod movegen;
mod search;
mod utils;
mod eval;

#[cfg(test)]
#[path = "../test/perft.rs"]
mod perft;

use crate::game::board::*;
use crate::game::load::*;
use crate::game::state::*;
use crate::movegen::generate_moves;
use crate::search::search;
use crate::search::perft;


fn main() {
    let (mut board, mut state) = load_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    // let (mut board, mut state) = load_fen("rnb2k1r/bp1P1ppp/2p5/q5n1/2B5/4B3/PPP1NKPP/RN1Q3R w - - 3 9");
    let moves = generate_moves(&board, &state);
    let mut count = 0u64;
    for m in moves {
        // if format!("{}", m) != "a7e3" {continue;}
        state = board.do_move(state, &m);
        print!("{}", m);
        
        // for i in 1..3  {
        //     // load_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        //     // println!("Search to depth: {}", i);
        //     
        //     let add = search(i, &mut board, state.clone());
        //     count += add;
        //     println!(": {}", add);
        // }

        let add = perft(4, &mut board, state.clone());
        count += add;
        println!(": {}", add);

        state = board.do_unmove(state, &m);
    }

    println!("Count: {}", count);
}
