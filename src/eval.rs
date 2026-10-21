use crate::game::{board::Board, state::State, datatypes::*};

use std::collections::HashMap;
use std::sync::LazyLock;

// ALL MOVE CALCULTIONS SHOULD BE HOW GOOD IT IS FOR PLAYER TO MOVE - NOT PLAYER WHO JUST MOVED
const CHECKMATE : i64 = i64::MAX;


static PIECE_VALUES: LazyLock<HashMap<PieceType, i64>> = LazyLock::new(|| {
    HashMap::from([
        (PieceType::KING, 10000000),
        (PieceType::PAWN, 100),
        (PieceType::KNIGHT, 300),
        (PieceType::BISHOP, 350),
        (PieceType::ROOK, 500),
        (PieceType::QUEEN, 900),
    ])
}); 

fn score_pieces(board: &Board, state: &State) -> i64 {
    let mut val : i64 = 0;

    for piece in board.pieces() {
        val += (if (piece.color as u8) == (state.turn() as u8) { 1 } else { -1 }) * PIECE_VALUES[&piece.piece_type];
    }

    val
}

fn score_positions() -> i64 {
    0
}

pub fn score_mobility() -> i64 {
    0
}

pub fn eval(board: &Board, state: &State) -> i64 {
    // Check for checkmate
    score_pieces(board, state)
}
