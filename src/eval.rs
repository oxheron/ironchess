use crate::game::{board::Board, state::State, datatypes::*};

use std::collections::HashMap;
use std::sync::LazyLock;

// ALL MOVE CALCULTIONS SHOULD BE HOW GOOD IT IS FOR PLAYER TO MOVE - NOT PLAYER WHO JUST MOVED
// Keep mate scores safely negatable in an alpha-beta window.
pub const MATE: i32 = 1_000_000;
pub const INF: i32 = MATE + 1;

static PIECE_VALUES: LazyLock<HashMap<PieceType, i32>> = LazyLock::new(|| {
    HashMap::from([
        (PieceType::KING, 10000000),
        (PieceType::PAWN, 100),
        (PieceType::KNIGHT, 300),
        (PieceType::BISHOP, 350),
        (PieceType::ROOK, 500),
        (PieceType::QUEEN, 900),
    ])
}); 

fn score_pieces(board: &Board, state: &State) -> i32 {
    let mut val : i32 = 0;

    // Empty squares can contain stale piece records after make/unmake.
    // Occupancy is the source of truth for which records to evaluate.
    let mut occupied = board.all();
    while occupied != 0 {
        let square = occupied.trailing_zeros() as usize;
        let piece = board.pieces()[square];
        val += (if (piece.color as u8) == (state.turn() as u8) { 1 } else { -1 }) * PIECE_VALUES[&piece.piece_type];
        occupied &= occupied - 1;
    }

    val
}

fn score_positions() -> i32 {
    0
}

pub fn score_mobility() -> i32 {
    0
}

pub fn eval(board: &Board, state: &State) -> i32 {
    // Check for checkmate
    score_pieces(board, state)
}
