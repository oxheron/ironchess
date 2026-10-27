use super::zobrist_from_board;
use crate::game::{board::Board, load::load_fen, state::State};
use crate::movegen::generate_moves;
use crate::zobrist;

// Quiet moves, double pushes, captures, castling, en passant, and all four
// promotion types (with and without captures), for both colors.
const POSITIONS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
    "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
    "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 2",
    "4k3/8/8/8/3Pp3/8/8/4K3 b - d3 0 2",
    "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
    "4k3/8/8/8/8/8/p7/1R2K3 b - - 0 1",
];

#[test]
fn loaded_hash_matches_full_recomputation() {
    for fen in POSITIONS {
        let (board, state) = load_fen(fen);
        assert_eq!(state.zobrist(), zobrist_from_board(&board, &state), "FEN: {fen}");
    }
}

#[test]
fn move_hash_changes_match_full_recomputation() {
    for fen in POSITIONS {
        let (board, state) = load_fen(fen);
        let initial_hash = zobrist_from_board(&board, &state);
        for cmove in generate_moves(&board, &state) {
            let mut moved_board = board.clone();
            let moved_state = moved_board.do_move(state.clone(), &cmove);
            // Compare XOR deltas to check move updates independently of loading.
            assert_eq!(
                moved_state.zobrist() ^ state.zobrist(),
                zobrist_from_board(&moved_board, &moved_state) ^ initial_hash,
                "FEN: {fen}, move: {cmove}",
            );
        }
    }
}

#[test]
fn undo_restores_hash_and_state() {
    for fen in POSITIONS {
        let (board, state) = load_fen(fen);
        for cmove in generate_moves(&board, &state) {
            let mut moved_board = board.clone();
            let moved_state = moved_board.do_move(state.clone(), &cmove);
            let restored = moved_board.do_unmove(moved_state, &cmove);
            assert_eq!(
                restored.zobrist(),
                state.zobrist(),
                "FEN: {fen}, move: {cmove}"
            );
            assert_eq!(
                restored.epfile(),
                state.epfile(),
                "FEN: {fen}, move: {cmove}"
            );
            assert_eq!(restored.castling(), state.castling());
            assert_eq!(restored.capture_type(), state.capture_type());
            assert_eq!(restored.fiftymove(), state.fiftymove());
            assert_eq!(restored.halfmoves(), state.halfmoves());
            assert_eq!(moved_board.all(), board.all());
            assert_eq!(moved_board.colored(), board.colored());
            assert_eq!(moved_board.piece(), board.piece());
        }
    }
}
