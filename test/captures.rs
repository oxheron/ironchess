use super::generate_mvv_lva_bitboards;
use crate::game::{
    datatypes::{Move, PieceType},
    load::load_fen,
};
use crate::movegen::{generate_info, generate_move_bitset};
use crate::search::memory::SearchFrame;
use crate::test_support::capture_moves;

fn capture_notation(fen: &str) -> Vec<String> {
    let (board, state) = load_fen(fen);
    capture_moves(&board, &state)
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn captures_order_victims_before_attackers_and_cover_every_piece_type() {
    // All six white piece types attack d4. Captures of the remaining victim
    // types must come later, even when a cheaper attacker is available.
    assert_eq!(
        capture_notation("7k/8/8/p5n1/R2qKb1r/4PN2/1B6/3Q4 w - - 0 1"),
        [
            "e3d4", "f3d4", "b2d4", "a4d4", "d1d4", "e4d4", "f3h4", "e3f4", "e4f4", "f3g5", "a4a5",
        ]
    );
}

#[test]
fn en_passant_is_appended_for_both_colors_and_requires_an_ep_file() {
    for (fen, expected) in [
        (
            "7k/8/8/p1PpP3/R7/8/8/K7 w - d6 0 1",
            vec!["a4a5", "c5d6", "e5d6"],
        ),
        (
            "k7/8/8/r7/P1pPp3/8/8/7K b - d3 0 1",
            vec!["a5a4", "c4d3", "e4d3"],
        ),
        ("7k/8/8/pP6/8/8/8/K7 w - a6 0 1", vec!["b5a6"]),
        ("7k/8/8/pP6/8/8/8/K7 w - - 0 1", vec![]),
    ] {
        assert_eq!(capture_notation(fen), expected, "{fen}");
    }
}

#[test]
fn promotion_membership_tracks_the_destination_independently_of_piece_type() {
    let (board, state) = load_fen("1r5k/P7/8/8/8/8/8/7K w - - 0 1");
    assert_eq!(
        generate_mvv_lva_bitboards(&board, &state),
        [(48, 1u64 << 57)]
    );

    let mut frame = SearchFrame::default();
    assert!(frame.move_bitset.is_empty());
    generate_move_bitset(&board, &state, &generate_info(&board, &state), &mut frame);
    assert_eq!(
        frame.move_bitset.get_square(48),
        (1u64 << 56) | (1u64 << 57)
    );
    let promotion = Move {
        start: 48,
        end: 57,
        piece: PieceType::QUEEN,
    };
    for piece in [
        PieceType::QUEEN,
        PieceType::KNIGHT,
        PieceType::ROOK,
        PieceType::BISHOP,
    ] {
        assert!(frame.move_bitset.check_move(&Move { piece, ..promotion }));
    }
    assert!(frame.move_bitset.clear_move(&promotion));
    assert!(!frame.move_bitset.clear_move(&promotion));
    for piece in [
        PieceType::QUEEN,
        PieceType::KNIGHT,
        PieceType::ROOK,
        PieceType::BISHOP,
    ] {
        assert!(!frame.move_bitset.check_move(&Move { piece, ..promotion }));
    }
    assert_eq!(frame.move_bitset.get_square(48), 1u64 << 56);
    frame.move_bitset.clear();
    assert!(frame.move_bitset.is_empty());
}
