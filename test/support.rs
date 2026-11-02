use crate::game::{board::Board, datatypes::*, state::State};
use crate::movegen::captures::generate_mvv_lva_bitboards;
use crate::search::{generate_ordered_moves, memory::SearchFrame};

pub fn legal_moves(board: &Board, state: &State) -> Vec<Move> {
    let mut frame = SearchFrame::default();
    generate_ordered_moves(board, state, &mut frame);
    frame.ordered_moves
}

// Capture generation supplies destinations; promotion expansion belongs to ordering.
pub fn capture_moves(board: &Board, state: &State) -> Vec<Move> {
    let mut moves = Vec::new();
    for (start, mut destinations) in generate_mvv_lva_bitboards(board, state) {
        while destinations != 0 {
            moves.push(Move {
                start,
                end: destinations.trailing_zeros() as u8,
                piece: PieceType::EMPTY,
            });
            destinations &= destinations - 1;
        }
    }
    moves
}
