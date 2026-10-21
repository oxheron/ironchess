use crate::dprintln;
use crate::movegen::generate_moves;
use crate::movegen::moveinfo::*;
use crate::game::board::*;
use crate::game::state::*;
use crate::eval::eval;

pub fn perft(depth: u32, board: &mut Board, mut state: State) -> u64
{
    // dprintln!("{:?}", depth);
    let mut count: u64 = 0;
    let moves = generate_moves(board, &state);

    if depth == 1 { 
        count += moves.len() as u64; 
        return count;
    }

    // dprintln!("{:?}", depth);
    for m in moves {
        // dprintln!("{:?}", m);
        state = board.do_move(state, &m);
        count += perft(depth - 1, board, state.clone());
        state = board.do_unmove(state, &m);
    }

    count
}

pub fn search(depth: u32, board: &mut Board, mut state: State) -> i64 {
    if depth == 0 {
        return eval(board, &state) 
    }

    let moves = generate_moves(board, &state);

    if moves.len() == 0 {
        // Checkmate or stalemate.  
        if generate_info(board, &state).check {
            return i64::MIN; 
        }

        return 0;
    }

    if state.halfmoves() >= 150 {
        return 0;
    }

    let mut best_score: i64 = i64::MIN;

    for m in moves {
        state = board.do_move(state, &m);
        let score = -search(depth - 1, board, state.clone());
        state = board.do_unmove(state, &m);

        best_score = best_score.max(score)
    }

    return best_score
}
